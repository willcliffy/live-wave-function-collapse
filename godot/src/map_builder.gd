extends Node3D

@onready var driver = $LWFCDriver
@onready var slot_scene = preload("res://scenes/Slot.tscn")

var slot_matrix: Array = []

var changes_queued: Array = []

func _ready():
	$Area.mesh.size = driver.map_size
	$Area.position = floor(Vector3(driver.map_size) / 2) - Vector3.ONE * 0.5

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


func _process(_delta):
	for i in range(100):
		if len(changes_queued) > 0:
			var change = changes_queued.pop_front()
			var change_position = change[0]
			var slot = slot_matrix[change_position.y][change_position.x][change_position.z]
			if slot:
				slot.constrain(change[1])


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
		var change_protos: String = change["new_protos"]
		if not change_protos:
			continue
		var change_position: Vector3i = change["position"]
		changes_queued.append([change_position, change_protos.split(",")])


func _on_slots_changed(changes):
	print("got slot change! ", changes)
