extends SubViewportContainer

signal slot_selected(slot: Area3D)

var slot_scene = preload("res://scenes/Slot.tscn")

var auto_collapsing = false

var map_size: Vector3
var slot_matrix = []
var slots = []


var wave_function: WfcCollapser.WaveFunction

var last_collapsed = []


func _ready():
	wave_function = WfcCollapser.WaveFunction.new()


func _process(delta):
	wave_function.process(delta)
	$Label.text = "FPS: " + str(Engine.get_frames_per_second())


func generate_slots(input_map_size: Vector3):
	map_size = input_map_size

	var x_offset = map_size.x / 2
	var z_offset = map_size.z / 2
	$Viewport/Scene/CameraBase.position += Vector3(x_offset, 0, z_offset)
	
	wave_function.initialize(map_size)
	wave_function.superposition_collapsed.connect(
		func(superposition: WfcCollapser.Superposition):
			# print("superposition at ", superposition.position, " collapsed to ", superposition.collapsed_to)
			var slot = _get_slot(superposition.position)
			slot.collapse(superposition.collapsed_to)
			slot.set_last_collapsed()
			last_collapsed.push_front(slot)
			if len(last_collapsed) > 3:
				last_collapsed.pop_back().clear_last_collapsed()
	)
	wave_function.superposition_constrained.connect(
		func(superposition: WfcCollapser.Superposition):
			#print("superposition at ", superposition.position, " constrained to entropy ", len(superposition.get_possibilities()))
			_get_slot(superposition.position).constrain(superposition.get_possibilities())
	)
	wave_function.superposition_overcollapsed.connect(
		func(position: Vector3):
			_get_slot(position).overconstrained()
	)

	for y in range(map_size.y):
		slot_matrix.append([])
		for x in range(map_size.x):
			slot_matrix[y].append([])
			for z in range(map_size.z):
				var slot = slot_scene.instantiate()
				slot.position = Vector3(x, y, z)
				slot.selected.connect(
					func():
						for s in slots:
							if s.name != slot.name:
								s.deselect()
						if not auto_collapsing:
							slot_selected.emit(slot)
				)
				slot.expand(WfcCollapser.WFCUtils.proto_data.keys())
				$Viewport/Scene.add_child(slot)
				slots.append(slot)
				slot_matrix[y][x].append(slot)
				slot.z_changed(0)

	return slots


func _on_apply_custom_constraints_pressed():
	wave_function.apply_custom_constraints()


func z_changed(value):
	for slot in slots:
		slot.z_changed(value)


func set_auto_collapse(button_pressed):
	auto_collapsing = button_pressed
	wave_function.autocollapsing = auto_collapsing


func toggle_axes(axes_visible): 
	$Viewport/Scene/Axes.visible = axes_visible


func toggle_zgrid(zgrid_visible):
	$Viewport/Scene/grid.visible = zgrid_visible


func _get_slot(position: Vector3) -> Area3D:
	return slot_matrix[position.y][position.x][position.z]


func collapse(position: Vector3, proto: String):
	wave_function.collapse(position, proto)
