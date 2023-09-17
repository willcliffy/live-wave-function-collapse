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
