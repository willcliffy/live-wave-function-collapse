extends SubViewportContainer

signal slot_selected(slot: Area3D)
signal slot_collapsed(slot: Area3D)

const CONSTRAIN_PER_TICK = 1
const COLLAPSE_PER_TICK = 5
const SELECT_RANDOM_LOWEST_ENTROPY = true

var slot_scene = preload("res://scenes/Slot.tscn")

var map_size

var slots = []

# 3D matrix: y -> x -> z
var slot_matrix = []

var constrained_slots_stack = []

var currently_selected
var last_collapsed

var auto_collapsing = false
var overcollapsed = false
var undoing = false

var history = []

enum ConstrainType {
	EXPLICIT_COLLAPSE,
	IMPLICIT_COLLAPSE,
	EXPLICIT_CONSTRAIN,
	IMPLICIT_CONSTRAIN
}

func new_collapse_step(
	slot: Area3D,
	constrained_neighbors: Array,
	constrain_type: ConstrainType = ConstrainType.IMPLICIT_COLLAPSE
) -> Dictionary:
	return {
		"type":                  constrain_type,
		"slot":                  slot.position,
		"collapsed_to":          slot.get_possibilities(),
		"remaining_proto":       slot.get_possibilities(true),
		"constrained_neighbors": constrained_neighbors
	}


func new_constrain_step(
	slot: Area3D,
	constrained_neighbors: Array,
	constrain_type: ConstrainType = ConstrainType.IMPLICIT_CONSTRAIN
) -> Dictionary:
	var removed = []
	for p in slot.get_last_possibilities():
		if p not in slot.get_possibilities():
			removed.append(p)
	return {
		"type":                  constrain_type,
		"slot":                  slot.position,
		"removed_proto":         removed,
		"remaining_proto":       slot.get_possibilities(),
		"constrained_neighbors": constrained_neighbors
	}


func _process(_delta):
	if overcollapsed:
		return

	if undoing:
		while undoing:
			if len(history) < 1:
				undoing = false
				return
			var current = history.pop_front()
			print(current)
		undoing = false
		return

	for i in range(10):
		if len(constrained_slots_stack) > 0:
			propagate(constrained_slots_stack.pop_front())
		elif auto_collapsing:
			_on_collapse_lowest_entropy_pressed()


func generate_slots(input_map_size: Vector3):
	map_size = input_map_size
	var x_offset = map_size.x / 2
	var z_offset = map_size.z / 2
	$Viewport/Scene/CameraBase.position += Vector3(x_offset, 0, z_offset)

	for y in range(map_size.y):
		slot_matrix.append([])
		for x in range(map_size.x):
			slot_matrix[y].append([])
			for z in range(map_size.z):
				var slot = slot_scene.instantiate()
				slot.position = Vector3(x, y, z)
				slot.selected.connect(on_slot_selected)
				slot.expand(WFC.proto_data.keys())
				slots.append(slot)
				slot_matrix[y][x].append(slot)
				$Viewport/Scene.add_child(slot)


func _on_apply_custom_constraints_pressed():
	# Make sure that there are no "uncapped" prototypes along the top of the space.

	var constrain_uncapped_slots_stack = []

	var top_level = slot_matrix[map_size.y - 1]
	for x in range(len(top_level)):
		var row = top_level[x]
		for z in range(len(row)):
			constrain_uncapped_slots_stack.append({
				"slot": row[z],
				"direction": Vector3.MODEL_TOP
			})

	for y in range(len(slot_matrix)):
		for x in range(len(slot_matrix[y])):
			for z in range(len(slot_matrix[y][x])):
				if x == 0:
					constrain_uncapped_slots_stack.append({
						"slot": slot_matrix[y][x][z],
						"direction": Vector3.MODEL_RIGHT
					})
				if x == map_size.x - 1:
					constrain_uncapped_slots_stack.append({
						"slot": slot_matrix[y][x][z],
						"direction": Vector3.MODEL_LEFT
					})
				if z == 0:
					constrain_uncapped_slots_stack.append({
						"slot": slot_matrix[y][x][z],
						"direction": Vector3.MODEL_REAR
					})
				if z == map_size.z - 1:
					constrain_uncapped_slots_stack.append({
						"slot": slot_matrix[y][x][z],
						"direction": Vector3.MODEL_FRONT
					})

	while len(constrain_uncapped_slots_stack) > 0:
		var slot_data = constrain_uncapped_slots_stack.pop_front()
		var direction = slot_data["direction"]
		var slot = slot_data["slot"]
		slot.constrain_uncapped(direction)
		constrained_slots_stack.append(slot)


func spawn_center_platform():
	var platform_y = 2
	var platform_x_i = 1.0 / 3.0 * map_size.x
	var platform_x_f = 2.0 / 3.0 * map_size.x
	var platform_z_i = 1.0 / 3.0 * map_size.z
	var platform_z_f = 2.0 / 3.0 * map_size.z
	for x in range(len(slot_matrix[platform_y])):
		if x < platform_x_i or x > platform_x_f:
			continue
		for z in range(len(slot_matrix[platform_y][x])):
			if z < platform_z_i or z > platform_z_f:
				continue
			slot_matrix[platform_y][x][z].collapse_default()


func _on_z_value_changed(value):
	$Viewport/Scene/Grid.position = Vector3(0, value, 0)
	for slot in slots:
		slot.z_changed(value)


func on_slot_selected(selected_slot: Area3D):
	for slot in slots:
		if slot.name != selected_slot.name:
			slot.deselect()
	currently_selected = selected_slot
	if not auto_collapsing:
		slot_selected.emit(selected_slot)


func propagate(constrained_slot: Area3D): # , constrain_type: ConstrainType = ConstrainType.EXPLICIT_CONSTRAIN):
	if constrained_slot.is_collapsed:
		set_last_collapsed(constrained_slot)

	var constrained_neighbors = []

	var neighbors = get_neighbors(constrained_slot.position)
	for neighbor in neighbors:
		if neighbor.is_collapsed: continue
		var new_neighbor_possibilities = []
		for constrained_proto in constrained_slot.get_possibilities():
			for neighbor_possibility in neighbor.get_possibilities():
				if neighbor_possibility in new_neighbor_possibilities: 
					continue
				if WFC.protos_compatible(
					constrained_proto,
					constrained_slot.position,
					neighbor_possibility,
					neighbor.position
				):
					new_neighbor_possibilities.append(neighbor_possibility)
		if len(new_neighbor_possibilities) != len(neighbor.get_possibilities()):
			if len(new_neighbor_possibilities) == 0:
				neighbor.overconstrained()
				auto_collapsing = false
				overcollapsed = true
			else:
				neighbor.constrain(new_neighbor_possibilities)
				constrained_neighbors.append(neighbor)
				constrained_slots_stack.append(neighbor)

	# TODO - Handle history outside of propagate
#	var history_entry
#	if constrained_slot.is_collapsed:
#		history_entry = new_collapse_step(
#			constrained_slot,
#			constrained_neighbor_positions
#		)
#	else:
#		history_entry = new_constrain_step(
#			constrained_slot,
#			constrained_neighbor_positions
#		)
#	history.append(history_entry)
#
#	# Do this last to preserve history ordering
#	for neighbor in constrained_neighbors:
#		propagate(neighbor)

	return constrained_neighbors


func toggle_axes(axes_visible): 
	$Viewport/Scene/Axes.visible = axes_visible


func toggle_zgrid(zgrid_visible):
	$Viewport/Scene/grid.visible = zgrid_visible


func get_neighbors(slot_position: Vector3):
	var directions = []

	if slot_position.x < map_size.x - 1:
		directions.append(slot_position + Vector3(1, 0, 0))
	if slot_position.x > 0:
		directions.append(slot_position + Vector3(-1, 0, 0))

	if slot_position.y < map_size.y - 1:
		directions.append(slot_position + Vector3(0, 1, 0))
	if slot_position.y > 0:
		directions.append(slot_position + Vector3(0, -1, 0))

	if slot_position.z < map_size.z - 1:
		directions.append(slot_position + Vector3(0, 0, 1))
	if slot_position.z > 0:
		directions.append(slot_position + Vector3(0, 0, -1))

	var result = []
	for direction in directions:
		result.append(slot_matrix[direction.y][direction.x][direction.z])

	return result


func _on_select_lowest_entropy_pressed() -> bool:
	var lowest_entropy_value = 99999
	var lowest_entropy_slots

	for slot in slots:
		var entropy = slot.get_entropy()
		if entropy <= 1 or entropy > lowest_entropy_value:
			continue
		if entropy < lowest_entropy_value:
			lowest_entropy_value = entropy
			lowest_entropy_slots = [slot]
		else:
			lowest_entropy_slots.append(slot)
	
	if not lowest_entropy_slots:
		return false

	var i = 0
	if SELECT_RANDOM_LOWEST_ENTROPY:
		i = randi() % len(lowest_entropy_slots)
	var lowest_entropy_slot = lowest_entropy_slots[i]
	lowest_entropy_slot.set_selected()
	return true


func _on_collapse_selected_pressed():
	if currently_selected != null:
		currently_selected.collapse()
		var constrained = propagate(currently_selected)
		constrained_slots_stack.append_array(constrained)
		# TODO - handle history for all propagation steps
#		while not constrained.is_empty():
#			var neighbor = constrained.pop_front()
#			constrained.append_array(propagate(neighbor))
		


func set_last_collapsed(slot: Area3D):
	if last_collapsed:
		last_collapsed.clear_last_collapsed()
	last_collapsed = slot
	last_collapsed.set_last_collapsed()


func _on_collapse_lowest_entropy_pressed():
	var selected = _on_select_lowest_entropy_pressed()
	if not selected:
		auto_collapsing = false
		print(history)
	_on_collapse_selected_pressed()


func set_auto_collapse(button_pressed):
	auto_collapsing = button_pressed
	overcollapsed = false


func _on_undo_pressed():
	undoing = true
