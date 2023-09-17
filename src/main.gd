extends Control

@onready var proto_preview_scene = preload("res://scenes/ProtoPreview.tscn")

@onready var MAP = $Container/Map
@onready var TOGGLES = $Container/Controls/ToggleMargins/Toggles
@onready var BUTTONS = $Container/Controls/ButtonsMargins/Buttons
@onready var PROTO_OPTIONS = $"Container/Options/Proto Selector/VBoxContainer"

const SIZE = Vector3(20, 5, 20)
const UI_SCALE_OVERRIDE =  Vector2(1, 1)

var current_previews = []


func _ready():
	scale = UI_SCALE_OVERRIDE
	MAP.generate_slots(SIZE)
	MAP._on_z_value_changed(0)
	TOGGLES.get_node("ZSelect/Z").max_value = SIZE.y - 1


func _on_map_slot_selected(selected_slot: Area3D):
	clear_options_pane()

	for possibility in selected_slot.get_possibilities():
		var preview = proto_preview_scene.instantiate()
		var button = Button.new()
		if possibility != "p-1":
			preview.set_proto(possibility)
			preview.pressed.connect(
				func():
					selected_slot.collapse(possibility)
					clear_options_pane()
					PROTO_OPTIONS.add_child(button)
			)
		button.add_child(preview)
		PROTO_OPTIONS.add_child(button)
		current_previews.append(preview)


func clear_options_pane():
	current_previews = []
	for child in PROTO_OPTIONS.get_children():
		PROTO_OPTIONS.remove_child(child)


func _on_check_button_toggled(_button_pressed):
	for preview in current_previews:
		preview.toggle_spinning()


func _on_z_level_changed(value):
	MAP._on_z_value_changed(value)
	TOGGLES.get_node("ZSelect/Value").text = str(value)


func _on_top_constraint_pressed():
	BUTTONS.get_node("CustomConstraints").disabled = true
	MAP._on_apply_custom_constraints_pressed()


func _on_center_platform_pressed():
	BUTTONS.get_node("Platform").disabled = true
	MAP.spawn_center_platform()


func _on_auto_collapse_toggled(button_pressed):
	MAP.set_auto_collapse(button_pressed)
	if button_pressed:
		clear_options_pane()
