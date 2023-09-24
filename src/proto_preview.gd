extends Button

var mesh: MeshInstance3D

var is_spinning = false


func _process(delta):
	if is_spinning:
		$ViewportContainer/Viewport/Scene/Content.rotate_y(delta)


func set_mesh(new_mesh: MeshInstance3D, mesh_rotation: Vector3):
	if mesh:
		mesh.queue_free()

	if new_mesh:
		mesh = MeshInstance3D.new()
		mesh.mesh = new_mesh.mesh
		mesh.rotation = mesh_rotation
		$ViewportContainer/Viewport/Scene/Content.add_child(mesh)


func set_proto(proto_name: String):
	if mesh:
		mesh.queue_free()

	var proto_datum =  WfcCollapser.WFCUtils.proto_data[proto_name]
	var mesh_instance = WFC.meshes.instantiate().get_node(proto_datum["mesh_name"])
	var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)

	mesh = MeshInstance3D.new()
	mesh.mesh = mesh_instance.mesh
	mesh.rotation = mesh_rotation
	$ViewportContainer/Viewport/Scene/Content.add_child(mesh)
	$Label.text = proto_name


func toggle_spinning():
	is_spinning = not is_spinning
	$ViewportContainer/Viewport/Scene/Content.rotation = Vector3.ZERO
