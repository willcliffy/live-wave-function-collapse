extends Control

@onready var controls = $Container/Controls

@export
var DEFAULT_MAP_SIZE = Vector3(10, 5, 10)

@export
var DEFAULT_MAP_CHUNK_SIZE = Vector3(10, 5, 10)

@export
var DEFAULT_CHUNK_OVERLAP = 2

@export
var AUTO_RUN = true


func _ready():
	controls.set_initialize_values(DEFAULT_MAP_SIZE)

	if not AUTO_RUN:
		return

	controls._on_initialize_pressed()

	var start_timer = Timer.new()
	start_timer.autostart = false
	start_timer.one_shot = true
	start_timer.timeout.connect(controls._on_start_pressed)
	add_child(start_timer)
	WFC.map_initialized.connect(start_timer.start)

	var collapse_timer = Timer.new()
	collapse_timer.autostart = false
	collapse_timer.one_shot = true
	collapse_timer.timeout.connect(controls._on_finalize_pressed)
	add_child(collapse_timer)
	WFC.map_collapsed.connect(collapse_timer.start)


func _on_map_viewer_reload_scene(_scene_filename):
	#get_tree().change_scene_to_file(scene_filename)
	pass
