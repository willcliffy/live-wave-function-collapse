extends Node

signal slot_constrained(slot: Vector3, protos: Array)
signal map_initialized
signal map_collapsed

const PROTO_FILE_NAME = "prototype_data.json"

var _proto_data: Dictionary
var _valid_neighbors: Dictionary

var _thread: Thread
var _collapser: WfcCollapser

var _autocollapse := false
var _autocollapse_started: float

const AUTOCOLLAPSE_SPEED = 5


func _ready():
	_load_proto_data()

	_thread = Thread.new()
	_collapser = WfcCollapser.new()
	_thread.start(_collapser.run)


func _process(_delta):
	if _autocollapse and _collapser.idle:
		var action := Action.new()
		action.type = ActionType.COLLAPSE
		_collapser.queue_action(action)


func _load_proto_data():
	const pX = 0
	const pY = 1
	const nX = 2
	const nY = 3
	const pZ = 4
	const nZ = 5

	if not FileAccess.file_exists(PROTO_FILE_NAME):
		print("File not found.")
		return

	var file = FileAccess.open(PROTO_FILE_NAME, FileAccess.READ)
	var json_text = file.get_as_text()
	file.close()

	_proto_data = JSON.parse_string(json_text)
	if typeof(_proto_data) != TYPE_DICTIONARY:
		print("Failed to parse JSON.")
		return

	_valid_neighbors = {}
	for proto in _proto_data:
		var proto_datum = _proto_data[proto]
		_valid_neighbors[proto] = {}
		_valid_neighbors[proto][Vector3.MODEL_TOP] = proto_datum["valid_neighbours"][pZ]
		_valid_neighbors[proto][Vector3.MODEL_BOTTOM] = proto_datum["valid_neighbours"][nZ]
		_valid_neighbors[proto][Vector3.MODEL_LEFT] = proto_datum["valid_neighbours"][pX]
		_valid_neighbors[proto][Vector3.MODEL_RIGHT] = proto_datum["valid_neighbours"][nX]
		_valid_neighbors[proto][Vector3.MODEL_FRONT] = proto_datum["valid_neighbours"][nY]
		_valid_neighbors[proto][Vector3.MODEL_REAR] = proto_datum["valid_neighbours"][pY]


func initialize(input_map_size: Vector3, input_map_chunk_size: Vector3, input_map_chunk_overlap: int):
	_collapser.initialize(input_map_size, input_map_chunk_size, input_map_chunk_overlap)


func _map_initialized():
	map_initialized.emit()


func _slot_constrained(slot: Vector3, protos: Array):
	slot_constrained.emit(slot, protos)


func start_collapse():
	_autocollapse = true
	_autocollapse_started = Time.get_unix_time_from_system()
	print(Time.get_datetime_string_from_system(), " autocollapse starting")


func stop_collapse():
	_autocollapse = false
	var elapsed = Time.get_unix_time_from_system() - _autocollapse_started
	print(Time.get_datetime_string_from_system(), " autocollapse stopped. Elapsed: ", elapsed)
	map_collapsed.emit()


# ---


class Slot:
	var position: Vector3
	var possibilities: Array
	var bucket: Array

	func expand(protos: Array):
		possibilities = protos
		_refresh_bucket()

	func collapse(proto: String = ""):
		if proto.is_empty():
			constrain([_choose_from_bucket()])
		else:
			constrain([proto])

	func constrain(protos: Array):
		possibilities = protos
		_refresh_bucket()

	func constrain_uncapped(direction: Vector3):
		var new_possibilities = []
		for proto in possibilities:
			if "p-1" in WFC._valid_neighbors[proto][direction]:
				new_possibilities.append(proto)
				new_possibilities.append(proto)

		if len(new_possibilities) != len(possibilities):
			constrain(new_possibilities)

	func remove_all(to_remove: Array):
		var new_possibilities = []
		for proto in possibilities:
			if not proto in to_remove:
				new_possibilities.append(proto)
		if len(new_possibilities) != len(possibilities):
			constrain(new_possibilities)

	func entropy() -> int:
		return len(possibilities)

	func is_collapsed() -> bool:
		return entropy() <= 1

	func _refresh_bucket():
		bucket = []
		for proto in possibilities:
			var weight = WFC._proto_data[proto]["weight"]
			for i in range(weight):
				bucket.append(proto)

	func _choose_from_bucket():
		return bucket[randi() % len(bucket)]


enum ActionType {
	INITIALIZE = 1,
	COLLAPSE = 2,
}


class Action:
	var type: ActionType
	var data: Variant


class WfcCollapser:
	# thread i/o
	var idle := false
	var _stop := false
	var _runner := Semaphore.new()
	var _queued_actions := []

	# game map data
	var map_size: Vector3
	var map_chunk_size: Vector3
	var map_chunk_overlap: float

	var slot_matrix = []
	var slots = []

	func initialize(input_map_size: Vector3, input_map_chunk_size: Vector3, input_map_chunk_overlap: int):
		map_size = input_map_size
		map_chunk_size = input_map_chunk_size
		map_chunk_overlap = input_map_chunk_overlap

		var action := Action.new()
		action.type = ActionType.INITIALIZE
		queue_action(action)

	func queue_action(action: Action):
		_queued_actions.push_front(action)
		_runner.post()

	func run():
		while true:
			idle = true
			_runner.wait()
			idle = false

			if _stop: break

			if len(_queued_actions) <= 0:
				print("Posted but no action queued!")
				continue

			var action = _queued_actions.pop_back()
			if action.type == ActionType.INITIALIZE:
				_generate_slots()
				_apply_custom_constraints()
				WFC._map_initialized.call_deferred()
			elif action.type == ActionType.COLLAPSE:
				for i in range(AUTOCOLLAPSE_SPEED):
					var ok = _collapse_next()
					if not ok: break
			else:
				print("Invalid action queued, skipping: ", action.type)

	func stop():
		_stop = true
		_runner.post()

	func _generate_slots():
		for y in range(map_size.y):
			slot_matrix.append([])
			for x in range(map_size.x):
				slot_matrix[y].append([])
				for z in range(map_size.z):
					var slot = Slot.new()
					slot.position = Vector3(x, y, z)
					slot.expand(WFC._proto_data.keys())
					slots.append(slot)
					slot_matrix[y][x].append(slot)

	func _apply_custom_constraints():
		# Only allowed protos on the bottom are sand and empty
		var constrained_to_bottom = []
		for proto in WFC._proto_data:
			if WFC._proto_data[proto]["constrain_to"] == "BOT":
				constrained_to_bottom.append(proto)

		# no "uncapped" prototypes along the sides of the space
		for y in range(map_size.y):
			for x in range(map_size.x):
				for z in range(map_size.z):
					var slot = slot_matrix[y][x][z]
					if y == 0:
						slot.constrain_uncapped(Vector3.MODEL_BOTTOM)
						_propagate(slot)
					else:
						slot.remove_all(constrained_to_bottom)
						_propagate(slot)

					if y == map_size.y - 1:
						slot.constrain_uncapped(Vector3.MODEL_TOP)
						_propagate(slot)
					if x == 0:
						slot.constrain_uncapped(Vector3.MODEL_RIGHT)
						_propagate(slot)
					if x == map_size.x - 1:
						slot.constrain_uncapped(Vector3.MODEL_LEFT)
						_propagate(slot)
					if z == 0:
						slot.constrain_uncapped(Vector3.MODEL_REAR)
						_propagate(slot)
					if z == map_size.z - 1:
						slot.constrain_uncapped(Vector3.MODEL_FRONT)
						_propagate(slot)

	func _collapse_next() -> bool:
		var selected = _select_lowest_entropy()
		if selected == null:
			print("selected null, toggling autocollapse")
			WFC.stop_collapse.call_deferred()
			return false

		selected.collapse()
		WFC._slot_constrained.call_deferred(selected.position, selected.possibilities)
		_propagate(selected)
		return true

	func _select_lowest_entropy() -> Slot:
		var lowest_entropy_value = 99999
		var lowest_entropy_slots

		for slot in slots:
			var entropy = slot.entropy()
			if entropy <= 1 or entropy > lowest_entropy_value:
				continue

			entropy += slot.position.y
			if slot.position.y == map_size.y:
				entropy += 2
			
			if entropy < lowest_entropy_value:
				lowest_entropy_value = entropy
				lowest_entropy_slots = [slot]
			else:
				lowest_entropy_slots.append(slot)

		if not lowest_entropy_slots:
			return null

		return lowest_entropy_slots[randi() % len(lowest_entropy_slots)]

	func _get_neighbors(slot_position: Vector3):
		var directions = []

		if slot_position.x < map_size.x - 1:
			directions.append(slot_position + Vector3(1, 0, 0))
		if slot_position.x > 0:
			directions.append(slot_position + Vector3(-1, 0, 0))

		if slot_position.y < map_size.y - 1:
			directions.append(slot_position + Vector3(0, 1, 0))
		if slot_position.y > 0:
			directions.append(slot_position + Vector3(0, -1, 0))

		if slot_position.z < map_size.z - 1:
			directions.append(slot_position + Vector3(0, 0, 1))
		if slot_position.z > 0:
			directions.append(slot_position + Vector3(0, 0, -1))

		var result = []
		for direction in directions:
			result.append(slot_matrix[direction.y][direction.x][direction.z])

		return result

	func _propagate(slot: Slot):
		var incomplete = _propagate_recursive(slot)
		while len(incomplete) > 0:
			var current = incomplete.pop_front()
			var inner_incomplete = _propagate_recursive(current)
			if len(inner_incomplete) > 0:
				print("Warning! Maxed call stack at least twice! Adding ", len(inner_incomplete), " additional propagations to queue of length ", len(incomplete))
				incomplete.append_array(inner_incomplete)

	func _propagate_recursive(slot: Slot, depth: int = 0):
		var incomplete = [] # Slots that should be propagated, but we can't without hitting recursion limit
		for neighbor in _get_neighbors(slot.position):
			if neighbor.is_collapsed(): continue
			var new_neighbor_possibilities = []
			for neighbor_proto in neighbor.possibilities:
				for proto in slot.possibilities:
					var direction = slot.position.direction_to(neighbor.position)
					if neighbor_proto in WFC._valid_neighbors[proto][direction]:
						new_neighbor_possibilities.append(neighbor_proto)
						break

			if len(new_neighbor_possibilities) != len(neighbor.possibilities):
				if len(new_neighbor_possibilities) == 0:
					print("overcollapsed!")
					WFC.stop_collapse.call_deferred()
					break

				neighbor.constrain(new_neighbor_possibilities)
				WFC._slot_constrained.call_deferred(neighbor.position, neighbor.possibilities)

				if depth >= 1000:
					incomplete.append(neighbor)
				else:
					incomplete.append_array(_propagate_recursive(neighbor, depth + 1))

		return incomplete


