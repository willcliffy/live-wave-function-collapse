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

	stop_thread = false
	runner = Semaphore.new()
	thread = Thread.new()
	thread.start(_wfc_collapser_run)


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

var map_size

var slot_matrix = []
var slots = []

func generate_slots(input_map_size: Vector3):
	map_size = input_map_size
	for y in range(map_size.y):
		slot_matrix.append([])
		for x in range(map_size.x):
			slot_matrix[y].append([])
			for z in range(map_size.z):
				var slot = Slot.new()
				slot.position = Vector3(x, y, z)
				slot.expand(proto_data.keys())
				slots.append(slot)
				slot_matrix[y][x].append(slot)


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


func select_lowest_entropy() -> Slot:
	var lowest_entropy_value = 99999
	var lowest_entropy_slots

	for slot in slots:
		var entropy = slot.entropy()
		if entropy <= 1 or entropy > lowest_entropy_value:
			continue
		if entropy < lowest_entropy_value:
			lowest_entropy_value = entropy
			lowest_entropy_slots = [slot]
		else:
			lowest_entropy_slots.append(slot)

	if not lowest_entropy_slots:
		return null

	return lowest_entropy_slots[randi() % len(lowest_entropy_slots)]


func propagate(slot: Slot, depth: int = 0):
	if slot == null:
		print("null slot!!")
	var neighbors = get_neighbors(slot.position)
	for neighbor in neighbors:
		if neighbor == null:
			print("null neighbor! ", neighbors)
		if neighbor.is_collapsed(): continue
		var new_neighbor_possibilities = []
		for proto in slot.possibilities:
			for neighbor_proto in neighbor.possibilities:
				if neighbor_proto in new_neighbor_possibilities: 
					continue
				if protos_compatible(
					proto,
					slot.position,
					neighbor_proto,
					neighbor.position
				):
					new_neighbor_possibilities.append(neighbor_proto)
		if len(new_neighbor_possibilities) != len(neighbor.possibilities):
			if len(neighbor.possibilities) < len(new_neighbor_possibilities):
				print("reduced neighbor ", neighbor.position, " to ", len(new_neighbor_possibilities), " from ", len(neighbor.possibilities))
			if len(new_neighbor_possibilities) == 0:
				autocollapse_toggled = true
				break

			neighbor.constrain(new_neighbor_possibilities)
			propagate(neighbor)
			constrained_slots_queued.append(neighbor)

			if depth > 1000:
				print("depth big ", depth)


# ---

class Slot:
	var position: Vector3
	var possibilities: Array

	func expand(protos: Array):
		possibilities = protos
	
	func collapse(proto: String = ""):
		if proto.is_empty():
			possibilities = [possibilities[randi() % len(possibilities)]]
		else:
			possibilities = [proto]

	func constrain(protos: Array):
		possibilities = protos

	func entropy() -> int:
		return len(possibilities)
	
	func is_collapsed() -> bool:
		return entropy() <= 1


signal slot_constrained(slot: Vector3, protos: Array)

const COLLAPSE_SPEED = 2

var runner: Semaphore
var thread: Thread
var stop_thread := false

var autocollapse := false
var autocollapse_toggled := false

var constrained_slots_queued = []


func _process(_delta):
	while len(constrained_slots_queued) > 0:
		var slot = constrained_slots_queued.pop_front()
		slot_constrained.emit(slot.position, slot.possibilities)

	if autocollapse:
		if autocollapse_toggled:
			autocollapse = false
			autocollapse_toggled = false
			auto_collapse_toggled.emit(false)
			print("autocollapse toggled!")
		runner.post()


func _wfc_collapser_run():
	while true:
		runner.wait()

		if stop_thread:
			break

		for i in range(COLLAPSE_SPEED):
			var selected = select_lowest_entropy()
			if selected == null:
				autocollapse_toggled = true
				return


			selected.collapse()
			constrained_slots_queued.append(selected)
			print("explicitly collapsed ", selected.position, " to ", selected.possibilities)
			propagate(selected)



func toggle_autocollapse(value):
	autocollapse = value


func wfc_stop():
	stop_thread = true
	runner.post()
	thread.wait_to_finish()
