extends Node3D

var _possibilities: Array = []
var _collapsed_to: String

var mesh

var highlight_enabled = false


func _ready():
	$Highlight.material_override = Preload.SlotMaterial.duplicate()


func collapse(proto_name: String = String()):
	if len(_possibilities) == 0:
		return # TODO - we should not overcollapse, but we should definitely not try to collapse an overcollapsed cell

	if proto_name.is_empty():
		proto_name = _possibilities[randi() % len(_possibilities)]

	_collapsed_to = proto_name

	play_constrain_animation()

	if proto_name == "-1" or proto_name == "p-1":
		return

	if mesh:
		mesh.visible = false
		remove_child(mesh)
		mesh = null

	var proto_datum = WFC._proto_data[proto_name]
	var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)
	var mesh_instance = Preload.ProtoMeshes.get_node(proto_datum["mesh_name"]).duplicate()
	mesh_instance.name = "Mesh"
	mesh_instance.rotation = mesh_rotation
	add_child(mesh_instance)
	mesh_instance.owner = self
	mesh = mesh_instance


func constrain(new_possibilities: Array):
	_possibilities = new_possibilities

	if len(_possibilities) == 1:
		collapse(_possibilities[0])
	elif len(_possibilities) == 0:
		overconstrained()
	else:
		play_constrain_animation()


func expand(new_possibilities: Array):
	if mesh and len(new_possibilities) > 1:
		mesh.visible = false
		remove_child(mesh)
		mesh = null

	_possibilities = new_possibilities
	play_expand_animation()


func overconstrained():
	if $Highlight.visible:
		$Highlight.material_override.set("shader_parameter/start_time",  0.0)
		$Highlight.material_override.set("shader_parameter/initial_color", Vector4(1, 0, 1, 0.5))

func play_constrain_animation():
	if $Highlight.visible:
		$Highlight.material_override.set("shader_parameter/start_time",  float(Time.get_ticks_msec()) / 1000.0)
		$Highlight.material_override.set("shader_parameter/initial_color", Vector4(1, 0, 0, 0.5))

func play_expand_animation():
	if $Highlight.visible:
		$Highlight.material_override.set("shader_parameter/start_time",  float(Time.get_ticks_msec()) / 1000.0)
		$Highlight.material_override.set("shader_parameter/initial_color", Vector4(0, 1, 0, 0.5))
