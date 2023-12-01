extends SubViewportContainer

signal reload_scene(scene_filename: String)


func _on_controls_initialize_map(input_map_size):
	$Viewport/Map.initialize_map(input_map_size)


func _get_all_children(in_node, arr := []):
	arr.push_back(in_node)
	for child in in_node.get_children():
		arr = _get_all_children(child,arr)
	return arr


func _on_controls_finalize_map():
	var slot_array = []
	for plane in $Viewport/Map.slot_matrix:
		for row in plane:
			for slot in row:
				slot_array.append(slot)

	var modules = preload("res://wfc_modules.glb").instantiate()
	var scene = preload("res://scenes/MapFinal.tscn").instantiate()

	for slot in slot_array:
		if slot._collapsed_to.is_empty() or slot._collapsed_to == "p-1":
			continue
		var proto_datum = WFC._proto_data[slot._collapsed_to]
		var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)
		var mesh_instance = modules.get_node(proto_datum["mesh_name"]).duplicate()
		mesh_instance.name = "%s" % [slot.position]
		mesh_instance.position = slot.position
		mesh_instance.rotation = mesh_rotation 
		scene.add_child(mesh_instance)
		mesh_instance.owner = scene
		for child in _get_all_children(mesh_instance):
			child.owner = scene

	scene.get_node("CameraBase").position = $Viewport/Map/CameraBase.position
	scene.get_node("CameraBase").rotation = $Viewport/Map/CameraBase.rotation
	var packed_scene = PackedScene.new()
	packed_scene.pack(scene)
	var map_filename = "res://_game/maps/map" + str(Time.get_unix_time_from_system()) + ".tscn"
	ResourceSaver.save(packed_scene, map_filename)

	#reload_scene.emit(scene_filename)

	$Viewport.add_child(scene)
	$Viewport/Map.queue_free()


func _on_controls_reset_map():
	pass
