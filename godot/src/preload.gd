extends Node

@onready var CellMaterial: ShaderMaterial = preload("res://resources/cell_highlight_material.tres")
@onready var ProtoMeshes: Node3D = preload("res://wfc_modules.glb").instantiate()


const PROTO_FILE_NAME = "prototype_data.json"
var ProtoData = {}


func _ready():
	if not FileAccess.file_exists(PROTO_FILE_NAME):
		print("File not found.")
		return

	var file = FileAccess.open(PROTO_FILE_NAME, FileAccess.READ)
	var json_text = file.get_as_text()
	file.close()

	ProtoData = JSON.parse_string(json_text)
	if typeof(ProtoData) != TYPE_DICTIONARY:
		print("Failed to parse JSON.")
		return
