extends Node3D

@onready var driver = $LWFCDriver
@onready var cell_scene = preload("res://scenes/Cell.tscn")

var cell_matrix: Array = []

var changes_queued: Array = []

func _ready():
	$CameraBase.position += Vector3(driver.map_size.x / 2, 0, driver.map_size.z / 2)
	for y in range(driver.map_size.y):
		cell_matrix.append([])
		for x in range(driver.map_size.x):
			cell_matrix[y].append([])
			for z in range(driver.map_size.z):
				var cell = cell_scene.instantiate()
				cell.name = "Cell %d %d %d" % [x, y, z]
				cell.position = Vector3(x, y, z)
				add_child(cell)
				cell.owner = self
				cell_matrix[y][x].append(cell)

	driver.start()


func _process(_delta):
	for i in range(100):
		if len(changes_queued) > 0:
			var change = changes_queued.pop_front()
			var change_position = change[0]
			var cell = cell_matrix[change_position.y][change_position.x][change_position.z]
			if cell:
				cell.change(change[1])


func play_expand_animation(cell_position: Vector3, protos: Array):
	var cell = cell_matrix[cell_position.y][cell_position.x][cell_position.z]
	if cell:
		cell.expand(protos)
	else:
		print("tried to expand null cell! ", cell_position)


func _on_cell_constrained(changes: Array):
	# TODO
	for raw_change in changes:
		var change = raw_change["CellChangeGodot"]
		var change_position: Vector3i = change["position"]
		var change_protos: String = change["new_protos"]
		changes_queued.append([change_position, change_protos.split(",")])


func _on_cells_changed(changes):
	print("got cell change! ", changes)
