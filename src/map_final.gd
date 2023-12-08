@tool
extends Node3D


func _ready():
	$CliffScatter.rebuild.emit()

