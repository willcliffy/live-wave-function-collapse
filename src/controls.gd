extends Control

signal initialize_map(size: Vector3)
signal start_collapse # unused
signal finalize_map
signal reset_map

@onready var size_selector = get_node("Controls/TabContainer/Setup/VBoxContainer/InitializeValues")
@onready var initialize_button: Button = get_node("Controls/TabContainer/Setup/VBoxContainer/Initialize")
@onready var start_button: Button = get_node("Controls/TabContainer/Setup/VBoxContainer/Start")
@onready var finalize_button: Button = get_node("Controls/TabContainer/Setup/VBoxContainer/Finalize")


func _ready():
	WFC.map_initialized.connect(func(): start_button.disabled = false, CONNECT_DEFERRED)
	WFC.map_collapsed.connect(func(): finalize_button.disabled = false, CONNECT_DEFERRED)


func set_initialize_values(values: Vector3):
	size_selector.get_node("XValue").text = str(values.x)
	size_selector.get_node("YValue").text = str(values.y)
	size_selector.get_node("ZValue").text = str(values.z)


func _on_initialize_pressed():
	initialize_button.disabled = true
	var map_size = Vector3(
		int(size_selector.get_node("XValue").text),
		int(size_selector.get_node("YValue").text),
		int(size_selector.get_node("ZValue").text)
	)
	initialize_map.emit(map_size)


func _on_start_pressed():
	start_button.disabled = true
	WFC.start_collapse()
	start_collapse.emit() # unused


func _on_finalize_pressed():
	finalize_button.disabled = true
	finalize_map.emit()


func _on_reset_pressed():
	initialize_button.disabled = false
	start_button.disabled = true
	finalize_button.disabled = true
	reset_map.emit()
