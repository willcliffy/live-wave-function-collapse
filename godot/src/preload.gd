extends Node

@onready var SlotMaterial: ShaderMaterial = preload("res://resources/slot_highlight_material.tres")
@onready var ProtoMeshes: Node3D = preload("res://wfc_modules.glb").instantiate()
