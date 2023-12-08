extends Node3D

signal map_finalized(path: String)

@onready var slot_scene = preload("res://scenes/Slot.tscn")
@onready var map_final_scene = preload("res://scenes/MapFinal.tscn")
@onready var grass = $Grass


@export_range(0, 16.0)
var TILE_RESOLUTION = 10.0

@export_range(0, 2.0 * PI)
var MAX_ANGLE_NORMAL = 3.0 * PI / 4.0

@export_range(0, 2.0 * PI)
var MAX_ROTATION = 2.0 * PI

@export_range(0, PI)
var MAX_TILT = PI / 7.0

@export_range(0, 1.0)
var MAX_SCALE_DELTA = 0.1

@export_range(0, 1.0)
var SCALE = 0.25

@export_range(0, 1)
var CLIFF_DETECTION_SKIRT = 0.3

var map_params: WFCModels.MapParams
var slot_matrix: Array = []

var initializing := false
var last_position := Vector3.ZERO


func _ready():
	WFC.slot_constrained.connect(play_constrain_animation, CONNECT_DEFERRED)
	WFC.slot_reset.connect(play_expand_animation, CONNECT_DEFERRED)

func _process(_delta):
	if not initializing:
		return


func initialize_map(params: WFCModels.MapParams):
	map_params = params
	$Area.mesh.size = params.size
	$Area.position = floor(params.size / 2) - Vector3.ONE * 0.5
	$Area.visible = true

	$CameraBase.position += Vector3(map_params.size.x / 2, 0, map_params.size.z / 2)
	for y in range(map_params.size.y):
		slot_matrix.append([])
		for x in range(map_params.size.x):
			slot_matrix[y].append([])
			for z in range(map_params.size.z):
				slot_matrix[y][x].append(null)

	initializing = true
	WFC.initialize(params)


func play_constrain_animation(slot_position: Vector3, protos: Array):
	var slot = slot_matrix[slot_position.y][slot_position.x][slot_position.z]
	if slot:
		slot.constrain(protos)
	else:
		slot = slot_scene.instantiate()
		slot.name = "Slot " + str(slot_position.x) + " " + str(slot_position.y) + " " + str(slot_position.z)
		slot.position = Vector3(slot_position.x, slot_position.y, slot_position.z)
		add_child(slot)
		slot.owner = self
		slot_matrix[slot_position.y][slot_position.x][slot_position.z] = slot
		slot.constrain(protos)


func play_expand_animation(slot_position: Vector3, protos: Array):
	var slot = slot_matrix[slot_position.y][slot_position.x][slot_position.z]
	if slot:
		slot.expand(protos)
	else:
		print("tried expanding null slot at ", slot_position)
