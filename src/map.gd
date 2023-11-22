extends SubViewportContainer

@onready var slot_scene = preload("res://scenes/Slot.tscn")

var slot_matrix = []


func _ready():
	WFC.slot_constrained.connect(
		func(slot_position, protos):
			_get_slot(slot_position).constrain(protos)
	)

func generate_slots(map_size: Vector3):
	var x_offset = map_size.x / 2
	var z_offset = map_size.z / 2
	$Viewport/Scene/CameraBase.position += Vector3(x_offset, 0, z_offset)

	WFC.generate_slots(map_size)
	for y in range(map_size.y):
		slot_matrix.append([])
		for x in range(map_size.x):
			slot_matrix[y].append([])
			for z in range(map_size.z):
				var slot = slot_scene.instantiate()
				slot.position = Vector3(x, y, z)
				$Viewport/Scene.add_child(slot)
				slot_matrix[y][x].append(slot)

func _on_apply_custom_constraints_pressed():
	WFC.apply_custom_constraints()

func set_auto_collapse(button_pressed):
	WFC.toggle_autocollapse(button_pressed)


func toggle_axes(axes_visible): 
	$Viewport/Scene/Axes.visible = axes_visible

func toggle_zgrid(zgrid_visible):
	$Viewport/Scene/grid.visible = zgrid_visible


func _get_slot(slot_position: Vector3):
	return slot_matrix[slot_position.y][slot_position.x][slot_position.z]
