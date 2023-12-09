extends MeshInstance3D

var fade_duration = 2.0
var current_time = 0.0
var fading_in = true


func _process(delta):
	current_time += delta

	# Calculate alpha value based on time and fade direction
	var alpha = clamp(current_time / fade_duration, 0.0, 1.0)
	if !fading_in:
		alpha = 1.0 - alpha

	# Update material's transparency
	var material = get_material_override()
	if material:
		var new_material = material.duplicate()
		var current_color = new_material.albedo_color
		new_material.albedo_color = Color(current_color.r, current_color.g, current_color.b, alpha)
		set_material_override(new_material)

	# Change fade direction when the fade is complete
	if current_time > fade_duration:
		current_time = 0.0
		fading_in = !fading_in
