extends SubViewportContainer

signal slot_selected(slot: Area3D)

var auto_collapsing = false


func _ready():
	WFC.auto_collapse_toggled.connect(
		func(value):
			auto_collapsing = value
	)


func _process(_delta):
	if auto_collapsing and WFC.is_ready():
		WFC.collapse()


func generate_slots(map_size: Vector3):
	var x_offset = map_size.x / 2
	var z_offset = map_size.z / 2
	$Viewport/Scene/CameraBase.position += Vector3(x_offset, 0, z_offset)
	
	var slots = WFC.generate_slots(map_size)
	for slot in slots:
		$Viewport/Scene.add_child(slot)
		slot.selected.connect(
			func():
				if not auto_collapsing:
					slot_selected.emit(slot)
		)


func _on_apply_custom_constraints_pressed():
	WFC.apply_custom_constraints()


func z_changed(value):
	WFC.z_changed(value)


func set_auto_collapse(button_pressed):
	auto_collapsing = button_pressed


func toggle_axes(axes_visible): 
	$Viewport/Scene/Axes.visible = axes_visible


func toggle_zgrid(zgrid_visible):
	$Viewport/Scene/grid.visible = zgrid_visible

