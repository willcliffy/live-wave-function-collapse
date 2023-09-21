extends Node

@onready var meshes = preload("res://wfc_modules.glb")

const PROTO_FILE_NAME = "prototype_data.json"

var proto_data

const pX = 0
const pY = 1
const nX = 2
const nY = 3
const pZ = 4
const nZ = 5


func _ready():
	if not FileAccess.file_exists(PROTO_FILE_NAME):
		print("File not found.")
		return

	var file = FileAccess.open(PROTO_FILE_NAME, FileAccess.READ)
	var json_text = file.get_as_text()
	file.close()

	proto_data = JSON.parse_string(json_text)
	if typeof(proto_data) != TYPE_DICTIONARY:
		print("Failed to parse JSON.")
		return

	exit_thread = false
	mutex = Mutex.new()
	runner = Semaphore.new()
	thread = Thread.new()
	thread.start(_wfc_collapser_run)
	
	map_mutex = Mutex.new()


func _direction_index(from: Vector3, to: Vector3, tolerance: float = 0.1) -> int:
	## NOTE that this function swaps Z and Y because of the shape of the proto data vs godot being Y-up
	if to.x > from.x + tolerance:
		return pX
	elif to.x < from.x - tolerance:
		return nX

	if to.y > from.y + tolerance:
		return pZ
	elif to.y < from.y - tolerance:
		return nZ

	if to.z > from.z + tolerance:
		return nY
	elif to.z < from.z - tolerance:
		return pY

	print(from, " ", to)
	return -1


func protos_compatible(proto, proto_position, other_proto, other_proto_position) -> bool:
	var direction_ind = _direction_index(proto_position, other_proto_position)
	return other_proto in proto_data[proto]["valid_neighbours"][direction_ind]


func proto_uncapped(proto: String, direction: Vector3 = Vector3.UP) -> bool:
	var direction_ind = _direction_index(Vector3.ZERO, direction)
	return not "p-1" in proto_data[proto]["valid_neighbours"][direction_ind]

# ---

signal auto_collapse_toggled(value: bool)

var slot_scene = preload("res://scenes/Slot.tscn")

var map_size

var currently_selected
var last_collapsed

var slot_matrix = []
var slots = []

var map_mutex: Mutex

func generate_slots(input_map_size: Vector3):
	map_size = input_map_size
	for y in range(map_size.y):
		slot_matrix.append([])
		for x in range(map_size.x):
			slot_matrix[y].append([])
			for z in range(map_size.z):
				var slot = slot_scene.instantiate()
				slot.position = Vector3(x, y, z)
				slot.selected.connect(
					func():
						currently_selected = slot
						for s in slots:
							if s.name != slot.name:
								s.deselect()
				)
				slot.expand(WFC.proto_data.keys())
				slots.append(slot)
				slot_matrix[y][x].append(slot)
	return slots


func apply_custom_constraints():
	# no "uncapped" prototypes along the top of the space
	for x in range(map_size.x):
		for z in range(map_size.z):
			var slot = slot_matrix[map_size.y - 1][x][z]
			slot.constrain_uncapped(Vector3.MODEL_TOP)
			propagate(slot.position, slot.get_possibilities())

	# no "uncapped" prototypes along the sides of the space
	for y in range(map_size.y):
		for x in range(map_size.x):
			for z in range(map_size.z):
				var slot = slot_matrix[y][x][z]
				if x == 0:
					slot.constrain_uncapped(Vector3.MODEL_RIGHT)
					propagate(slot.position, slot.get_possibilities())
				if x == map_size.x - 1:
					slot.constrain_uncapped(Vector3.MODEL_LEFT)
					propagate(slot.position, slot.get_possibilities())
				if z == 0:
					slot.constrain_uncapped(Vector3.MODEL_REAR)
					propagate(slot.position, slot.get_possibilities())
				if z == map_size.z - 1:
					slot.constrain_uncapped(Vector3.MODEL_FRONT)
					propagate(slot.position, slot.get_possibilities())


func z_changed(value):
	for slot in slots:
		slot.z_changed(value)


func get_neighbors(slot_position: Vector3):
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


func select_lowest_entropy() -> Area3D:
	var lowest_entropy_value = 99999
	var lowest_entropy_slots

	for slot in slots:
		var entropy = slot.get_entropy()
		if entropy <= 1 or entropy > lowest_entropy_value:
			continue
		if entropy < lowest_entropy_value:
			lowest_entropy_value = entropy
			lowest_entropy_slots = [slot]
		else:
			lowest_entropy_slots.append(slot)

	if not lowest_entropy_slots:
		return null

	var i = randi() % len(lowest_entropy_slots)
	var lowest_entropy_slot = lowest_entropy_slots[i]
	lowest_entropy_slot.set_selected()
	return lowest_entropy_slot


func propagate(constrained_position: Vector3, constrained_possibilities: Array, depth: int = 0):
	var neighbors = get_neighbors(constrained_position)
	for neighbor in neighbors:
		if neighbor.is_collapsed: continue
		var new_neighbor_possibilities = []
		for constrained_proto in constrained_possibilities:
			for neighbor_possibility in neighbor.get_possibilities():
				if neighbor_possibility in new_neighbor_possibilities: 
					continue
				if WFC.protos_compatible(
					constrained_proto,
					constrained_position,
					neighbor_possibility,
					neighbor._position
				):
					new_neighbor_possibilities.append(neighbor_possibility)
		if len(new_neighbor_possibilities) != len(neighbor.get_possibilities()):
			if len(new_neighbor_possibilities) == 0:
				neighbor.overconstrained()
				auto_collapse_toggled.emit(false)
				break

			neighbor.constrain(new_neighbor_possibilities)
			propagate(neighbor._position, new_neighbor_possibilities, depth + 1)
			if depth > 1000:
				print("depth big ", depth, " ", len(neighbor._possibilities) == len(neighbor.get_possibilities()), " ", len(new_neighbor_possibilities) - len(neighbor.get_possibilities()))


func set_last_collapsed(slot: Area3D):
	if last_collapsed:
		last_collapsed.clear_last_collapsed()
	last_collapsed = slot
	last_collapsed.set_last_collapsed()


# ---

var mutex: Mutex
var runner: Semaphore
var thread: Thread
var exit_thread := false
var working := true

var collapsed_slot: Area3D
var collapsed_slot_proto: String

func _wfc_collapser_run():
	while true:
		runner.wait() # Wait until posted.

		mutex.lock()
		var should_exit = exit_thread
		mutex.unlock()

		if should_exit:
			break

		mutex.lock()
		var proto = collapsed_slot._possibilities[randi() % len(collapsed_slot._possibilities)]
		collapsed_slot.collapse(proto)
		propagate(collapsed_slot._position, [proto])
		mutex.unlock()


func is_ready():
	if not mutex.try_lock():
		#print("not ready - still locked")
		return false

	mutex.unlock()
	return true


func collapse(slot: Area3D = null, proto: String = String()):
	mutex.lock()
	if slot == null:
		var slot_selected = select_lowest_entropy()
		if slot_selected == null:
			auto_collapse_toggled.emit(false)
			mutex.unlock()
			return
		collapsed_slot = slot_selected
	else:
		collapsed_slot = slot
	collapsed_slot_proto = proto
	mutex.unlock()
	runner.post()


func wfc_stop():
	mutex.lock()
	exit_thread = true
	mutex.unlock()
	runner.post()
	thread.wait_to_finish()
