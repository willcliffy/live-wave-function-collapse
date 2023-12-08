extends Node3D

@onready var Camera = $Camera

const CAMERA_ROTATION_SPEED = 65.0
const CAMERA_MOVEMENT_SPEED = 15.0


func _input(event):
	if event is InputEventMouseButton:
		handle_camera_zoom_input(event)


func _physics_process(delta):
	if Input.is_anything_pressed():
		handle_camera_rotation_input(delta)
		handle_camera_movement_input(delta)
		if Input.get_action_strength("aerial_camera_toggle") > 0:
			rotation_degrees = Vector3(-90, 0, 0)


func handle_camera_zoom_input(event):
	if event.button_index == MOUSE_BUTTON_WHEEL_UP and Camera.position.z > 5:
		Camera.position.z -= 1
	elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN and Camera.position.z < 500:
		Camera.position.z += 1


func handle_camera_movement_input(delta):
	var move_horizontal := Input.get_action_strength("move_right") - Input.get_action_strength("move_left")
	var move_vertical := Input.get_action_strength("move_back") - Input.get_action_strength("move_forward")
	position += CAMERA_MOVEMENT_SPEED * delta * Vector3(move_horizontal, 0, move_vertical).rotated(Vector3.UP, rotation.y)


func handle_camera_rotation_input(delta):
	var rotate_horizontal := Input.get_action_strength("rotate_right") - Input.get_action_strength("rotate_left")
	var rotate_vertical := Input.get_action_strength("rotate_down") - Input.get_action_strength("rotate_up")
	if rotate_horizontal != 0 or rotate_vertical != 0:
		var new_rotation = rotation_degrees
		new_rotation.y += rotate_horizontal * delta * CAMERA_ROTATION_SPEED
		new_rotation.x = new_rotation.x + rotate_vertical * delta * CAMERA_ROTATION_SPEED
		rotation_degrees = new_rotation

