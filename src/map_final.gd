@tool
extends Node3D

const SEALEVEL = -0.45

func _ready():
	var sea_mesh_width = 25
	var sea_mesh_length = 25
	var sea_width = 10
	var sea_length = 10

	$Ocean.multimesh.set_instance_count(sea_width * sea_length)
	for x in range(sea_width):
		for z in range(sea_width):
			var instance_x = (x - sea_width / 2.0) * sea_mesh_width
			var instance_z = (z - sea_length / 2.0) * sea_mesh_length
			var instance_transform := Transform3D(Basis(), Vector3(instance_x, SEALEVEL, instance_z))
			$Ocean.multimesh.set_instance_transform(x * sea_width + z, instance_transform)
	$CliffScatter.rebuild.emit()

