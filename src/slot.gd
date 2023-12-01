extends Node3D

@onready var meshes = preload("res://wfc_modules.glb").instantiate()

const CONSTRAIN_ANIMATION_DURATION = 0.5

var is_collapsed = false

var constrain_time_left = CONSTRAIN_ANIMATION_DURATION
var is_constraining = false
var is_expanding = false

var _possibilities: Array = []
var _collapsed_to: String


func _process(delta):
	if is_constraining:
		constrain_time_left -= delta
		var transparancy = 0.5 * constrain_time_left / CONSTRAIN_ANIMATION_DURATION
		$ConstrainHighlight.mesh.material.albedo_color = Color(1, 0, 0, transparancy)
		if constrain_time_left < 0:
			is_constraining = false
			$ConstrainHighlight.visible = false
	elif is_expanding:
		constrain_time_left -= delta
		var transparancy = 0.5 * constrain_time_left / CONSTRAIN_ANIMATION_DURATION
		$ExpandHighlight.mesh.material.albedo_color = Color(0, 1, 0, transparancy)
		if constrain_time_left < 0:
			is_expanding = false
			$ExpandHighlight.visible = false


func collapse(proto_name: String = String()):
	if len(_possibilities) == 0:
		return # TODO - we should not overcollapse, but we should definitely not try to collapse an overcollapsed cell

	if proto_name.is_empty():
		proto_name = _possibilities[randi() % len(_possibilities)]

	_collapsed_to = proto_name
	is_collapsed = true

	play_constrain_animation()

	if proto_name == "-1" or proto_name == "p-1":
		return

	var proto_datum = WFC._proto_data[proto_name]
	var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)
	var mesh_instance = meshes.get_node(proto_datum["mesh_name"]).duplicate()
	mesh_instance.name = "Mesh"
	mesh_instance.rotation = mesh_rotation
	add_child(mesh_instance)
	mesh_instance.owner = self


func constrain(new_possibilities: Array):
	_possibilities = new_possibilities

	if len(_possibilities) == 1:
		collapse(_possibilities[0])
	elif len(_possibilities) == 0:
		overconstrained()
	else:
		play_constrain_animation()


func expand(new_possibilities: Array):
	if is_collapsed and len(new_possibilities) > 1:
		is_collapsed = false

	_possibilities = new_possibilities
	play_expand_animation()


func overconstrained():
	$InvalidHighlight.visible = true


func play_constrain_animation():
	constrain_time_left = CONSTRAIN_ANIMATION_DURATION
	is_constraining = true
	$ConstrainHighlight.mesh.material.albedo_color = Color(1, 0, 0, 0.5)
	$ConstrainHighlight.visible = true


func play_expand_animation():
	constrain_time_left = CONSTRAIN_ANIMATION_DURATION
	is_expanding = true
	$ExpandHighlight.mesh.material.albedo_color = Color(0, 1, 0, 0.5)
	$ExpandHighlight.visible = true
