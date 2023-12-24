extends Node3D

var _possibilities: Array = []
var _collapsed_to: String

var mesh

var highlight_enabled = false


func _ready():
	$Highlight.material_override = Preload.CellMaterial.duplicate()


func change(new_possibilities: Array):
	if len(new_possibilities) == len(_possibilities):
		return # TODO - maybe have a visual indicator here anyways :think:

	if len(new_possibilities) == 1:
		collapse(new_possibilities[0])
		return

	if len(new_possibilities) == 0:
		overconstrained()
	elif len(new_possibilities) > len(_possibilities):
		expand(new_possibilities)
	else:
		play_constrain_animation()

	_possibilities = new_possibilities


func collapse(proto_name: String = String()):
	if proto_name.is_empty():
		overconstrained()
		return

	_collapsed_to = proto_name

	if mesh:
		mesh.visible = false
		remove_child(mesh)
		mesh = null

	play_constrain_animation()

	if proto_name == "-1" or proto_name == "p-1":
		return

	var proto_datum = Preload.ProtoData[proto_name]
	var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)
	var mesh_instance = Preload.ProtoMeshes.get_node(proto_datum["mesh_name"]).duplicate()
	mesh_instance.name = "Mesh"
	mesh_instance.rotation = mesh_rotation
	add_child(mesh_instance)
	mesh_instance.owner = self
	mesh = mesh_instance


func expand(new_possibilities: Array):
	if mesh and len(new_possibilities) > 1:
		mesh.visible = false
		remove_child(mesh)
		mesh = null

	_possibilities = new_possibilities
	play_expand_animation()


func overconstrained():
	if $Highlight.visible:
		$Highlight.material_override.set("shader_parameter/start_time",  INF)
		$Highlight.material_override.set("shader_parameter/initial_color", Vector4(1, 0, 1, 0.1))

func play_constrain_animation():
	if $Highlight.visible:
		$Highlight.material_override.set("shader_parameter/start_time",  float(Time.get_ticks_msec()) / 1000.0)
		$Highlight.material_override.set("shader_parameter/initial_color", Vector4(1, 0, 0, 0.5))

func play_expand_animation():
	if $Highlight.visible:
		$Highlight.material_override.set("shader_parameter/start_time",  float(Time.get_ticks_msec()) / 1000.0)
		$Highlight.material_override.set("shader_parameter/initial_color", Vector4(0, 1, 0, 0.5))
