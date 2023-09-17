extends Control

signal z_changed(z_value: int)
signal show_axes_toggled(show_axes: bool)
signal apply_custom_constraints()
signal auto_collapse_toggled(auto_collapsing: bool)
signal size_set(size: Vector3)

@onready var zselector = get_node("Controls/TabContainer/Collapse/Collapse/Toggles/ZSelect")
@onready var apply_custom_constraints_button = get_node("Controls/TabContainer/Setup/VBoxContainer/CustomConstraints")
@onready var size_selector = get_node("Controls/TabContainer/Setup/VBoxContainer/SizeSelector")
@onready var continue_button = get_node("Controls/TabContainer/Setup/VBoxContainer/Continue")
@onready var custom_constraints_button = get_node("Controls/TabContainer/Setup/VBoxContainer/CustomConstraints")


func _on_z_value_changed(value):
	zselector.get_node("Label").text = str(value)
	z_changed.emit(value)


func _on_show_axes_toggled(button_pressed):
	show_axes_toggled.emit(button_pressed)


func _on_apply_custom_constraints_pressed():
	apply_custom_constraints_button.disabled = true
	apply_custom_constraints.emit()


func _on_auto_collapse_toggled(button_pressed):
	auto_collapse_toggled.emit(button_pressed)


func _on_set_size_pressed():
	var x = int(size_selector.get_node("XValue").text)
	var y = int(size_selector.get_node("YValue").text)
	var z = int(size_selector.get_node("ZValue").text)
	zselector.get_node("Z").max_value = y
	size_set.emit(Vector3(x, y, z))

	# able to continue once Size is set
	custom_constraints_button.disabled = false
	continue_button.disabled = false


func _on_continue_pressed():
	$Controls/TabContainer.current_tab += 1
