extends Node3D

@onready var map_builder: Node3D = $MapBuilder


func _ready():
	print("%s Starting" % [Time.get_datetime_string_from_system()])

	#var start_timer = Timer.new()
	#start_timer.autostart = false
	#start_timer.one_shot = true
	#start_timer.timeout.connect(map_builder.start)
	#add_child(start_timer)
	#map_builder.map_initialized.connect(start_timer.start)

	#map_builder.start()
#
	#var collapse_timer = Timer.new()
	#collapse_timer.autostart = false
	#collapse_timer.one_shot = true
	#collapse_timer.timeout.connect(_finalize_map)
	#add_child(collapse_timer)
	#map_builder.map_completed.connect(collapse_timer.start)


func _get_all_children(in_node, arr := []):
	arr.push_back(in_node)
	for child in in_node.get_children():
		arr = _get_all_children(child,arr)
	return arr


func _finalize_map():
	var cell_array = []
	for plane in map_builder.cell_matrix:
		for row in plane:
			for cell in row:
				cell_array.append(cell)

	var modules = preload("res://wfc_modules.glb").instantiate()
	var scene = preload("res://scenes/MapFinal.tscn").instantiate()

	for cell in cell_array:
		if cell._collapsed_to.is_empty() or cell._collapsed_to == "p-1":
			continue

		var proto_datum = {} # WFC._proto_data[cell._collapsed_to]
		var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)
		var mesh_instance: MeshInstance3D = modules.get_node(proto_datum["mesh_name"]).duplicate()

		mesh_instance.name = "%s" % [cell.position]
		mesh_instance.position = cell.position
		mesh_instance.rotation = mesh_rotation 
		scene.add_child(mesh_instance)
		mesh_instance.owner = scene
		for child in _get_all_children(mesh_instance):
			child.owner = scene

	# scene.get_node("CliffScatter").shape_size = Vector3.ONE + DEFAULT_MAP_CHUNK_SIZE # TODO TODO TODO

	var source_camera_base := map_builder.get_node("CameraBase")
	var target_camera_base := scene.get_node("CameraBase")

	target_camera_base.position = source_camera_base.position
	target_camera_base.rotation = source_camera_base.rotation
	target_camera_base.get_node("Camera").position.z = source_camera_base.get_node("Camera").position.z

	var packed_scene = PackedScene.new()
	packed_scene.pack(scene)
	ResourceSaver.save(packed_scene, "res://_game/maps/map" + str(Time.get_unix_time_from_system()) + ".tscn")

	add_child(scene)
	map_builder.queue_free()
