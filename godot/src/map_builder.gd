extends Node3D

@onready var driver = $LWFCDriver
@onready var slot_scene = preload("res://scenes/Slot.tscn")

var slot_matrix: Array = []


func _ready():
	$Area.mesh.size = driver.map_size
	$Area.position = floor(Vector3(driver.map_size) / 2) - Vector3.ONE * 0.5
	$Area.visible = true

	$CameraBase.position += Vector3(driver.map_size.x / 2, 0, driver.map_size.z / 2)
	for y in range(driver.map_size.y):
		slot_matrix.append([])
		for x in range(driver.map_size.x):
			slot_matrix[y].append([])
			for z in range(driver.map_size.z):
				var slot = slot_scene.instantiate()
				slot.name = "Slot %d %d %d" % [x, y, z]
				slot.position = Vector3(x, y, z)
				add_child(slot)
				slot.owner = self
				slot_matrix[y][x].append(slot)

	driver.start()


func play_expand_animation(slot_position: Vector3, protos: Array):
	var slot = slot_matrix[slot_position.y][slot_position.x][slot_position.z]
	if slot:
		slot.expand(protos)
	else:
		print("tried to expand null slot! ", slot_position)


func _on_slot_constrained(changes: Array):
	# TODO
	for raw_change in changes:
		var change = raw_change["SlotChangeGodot"]
		var change_position: Vector3i = change["position"]
		var change_protos: String = change["new_protos"]
		var slot = slot_matrix[change_position.y][change_position.x][change_position.z]
		if slot:
			slot.constrain(change_protos.split(","))
	#else:
		#print("tried to constrain null slot! ", slot_position)


func _on_slots_changed(changes):
	print("got slot change! ", changes)
