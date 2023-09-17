extends Area3D

signal selected(slot: Area3D)
signal deselected(slot: Area3D)
signal collapsed(slot: Area3D)

const CONSTRAIN_ANIMATION_DURATION = 0.5

var is_selectable = false
var is_selected = false
var is_collapsed = false
var is_constraining = false
var is_expanding = false

var current_z: int = 0
var _last_possibilities: Array = []
var _possibilities: Array = []
var _collapsed_to: String

var constrain_time_left = CONSTRAIN_ANIMATION_DURATION


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

func _on_mouse_entered():
	if is_selectable and not is_selected:
		$HoveredHighlight.visible = true


func _on_mouse_exited():
	$HoveredHighlight.visible = false
	if is_selected:
		$SelectedHighlight.visible = true


func _on_input_event(_camera, event, _position, _normal, _shape_idx):
	if not event is InputEventMouseButton or not event.is_pressed(): return
	if event.button_index != MOUSE_BUTTON_LEFT: return
	if not is_selectable: return

	set_selected()


func z_changed(value):
	if is_selected:
		deselect()

	current_z = value
	is_selectable = position.y == value and not is_collapsed

	if is_selectable and not is_collapsed:
		$CollisionShape3D.disabled = false
		$HoveredHighlight.visible = false
	else:
		$CollisionShape3D.disabled = true


func set_selected():
	is_selected = true
	$SelectedHighlight.visible = true
	$HoveredHighlight.visible = false
	selected.emit(self)


func get_last_possibilities():
	return _last_possibilities

func get_possibilities(ignore_collapsed: bool = false):
	if is_collapsed and not ignore_collapsed:
		return [_collapsed_to]

	return _possibilities


func deselect():
	is_selected = false
	$SelectedHighlight.visible = false


func collapse_default():
	collapse("p8") # TODO - this should be GroundFlat


func collapse(proto_name: String = String()):
	if len(_possibilities) == 0:
		return # TODO - we should not overcollapse, but we should definitely not try to collapse an overcollapsed cell

	if proto_name.is_empty():
		proto_name = _possibilities[randi() % len(_possibilities)]

	_collapsed_to = proto_name
	is_collapsed = true
	is_selectable = false

	deselect()
	play_constrain_animation()
	# collapsed.emit(self)

	if proto_name == "-1" or proto_name == "p-1":
		return

	var proto_datum = WFC.proto_data[proto_name]
	var mesh_rotation = Vector3(0, proto_datum["mesh_rotation"] * PI/2, 0)
	var mesh_instance = WFC.meshes.instantiate().get_node(proto_datum["mesh_name"])
	var mesh = MeshInstance3D.new()
	mesh.mesh = mesh_instance.mesh
	mesh.rotation = mesh_rotation
	add_child(mesh)


func constrain(new_possibilities: Array):
	_last_possibilities = _possibilities
	_possibilities = new_possibilities

	if len(_possibilities) == 1:
		collapse(_possibilities[0])
		return

	play_constrain_animation()


func expand(new_possibilities: Array):
	if is_collapsed and len(new_possibilities) > 1:
		is_collapsed = false

	_last_possibilities = _possibilities
	_possibilities = new_possibilities

	play_expand_animation()


func overconstrained():
	_possibilities = []
	$InvalidHighlight.visible = true


func set_last_collapsed():
	$LastCollapsedHighlight.visible = true


func clear_last_collapsed():
	$LastCollapsedHighlight.visible = false


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


func get_entropy():
	if is_collapsed: return 1
	return len(_possibilities)


func constrain_uncapped(direction: Vector3 = Vector3.MODEL_TOP):
	var new_possibilities = []
	for proto in _possibilities:
		if not WFC.proto_uncapped(proto, direction):
			new_possibilities.append(proto)
	
	if len(new_possibilities) != len(_possibilities):
		_possibilities = new_possibilities
		if len(_possibilities) == 1:
			collapse(_possibilities[0])
		else:
			play_constrain_animation()
		
