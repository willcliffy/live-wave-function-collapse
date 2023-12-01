@tool
extends EditorPlugin

const Scatter := preload("./src/scatter.gd")

func _get_plugin_name():
	return "CliffScatter"


func _enter_tree():
	add_custom_type(
		"CliffScatter",
		"Node3D",
		preload("./src/scatter.gd"),
		preload("./icons/scatter.svg")
	)

	var editor_selection = get_editor_interface().get_selection()


func _exit_tree():
	remove_custom_type("CliffScatter")

