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


var slot_matrix = []

var last_position = Vector2(0, 0)
var initializing = false

var map_size: Vector3
var map_chunk_size: Vector3
var map_chunk_overlap: int


func _process(_delta):
	if not initializing:
		return

	if last_position.y == map_size.y:
		initializing = false
		print(Time.get_datetime_string_from_system(), " done creating visual slots")
		WFC.slot_constrained.connect(play_constrain_animation, CONNECT_DEFERRED)
		WFC.initialize(map_size, map_chunk_size, map_chunk_overlap)
		return

	for z in range(map_size.z):
		var slot = slot_scene.instantiate()
		slot.name = "Slot " + str(last_position.x) + " " + str(last_position.y) + " " + str(z)
		slot.position = Vector3(last_position.x, last_position.y, z)
		add_child(slot)
		slot.owner = self
		slot_matrix[last_position.y][last_position.x][z] = slot
		slot.play_expand_animation()

	last_position.x += 1
	if last_position.x == map_size.x:
		last_position.x = 0
		last_position.y += 1


func initialize_map(input_map_size: Vector3, input_map_chunk_size: Vector3, input_map_chunk_overlap: int):
	map_size = input_map_size
	map_chunk_size = input_map_chunk_size
	map_chunk_overlap = input_map_chunk_overlap

	$CameraBase.position += Vector3(map_size.x / 2, 0, map_size.z / 2)
	for y in range(map_size.y):
		slot_matrix.append([])
		for x in range(map_size.x):
			slot_matrix[y].append([])
			for z in range(map_size.z):
				slot_matrix[y][x].append(null)
	initializing = true


func play_constrain_animation(slot_position: Vector3, protos: Array):
	var slot = slot_matrix[slot_position.y][slot_position.x][slot_position.z]
	if slot:
		slot.constrain(protos)

