extends Node

class_name WFCChunk


class MapChunk:
	var map_params: WFCModels.MapParams
	var position: Vector3

	var _all_slots_array: Array = []
	var _all_slots_matrix: Array = []

	var _chunk_slots_set: Dictionary = {}
	var _chunk_slots_matrix: Array = []

	func initialize(params: WFCModels.MapParams, chunk_position: Vector3, slots_matrix: Array, slots_array: Array):
		map_params = params
		position = chunk_position
		_all_slots_array = slots_array
		_all_slots_matrix = slots_matrix

		for y in range(params.size.y):
			_chunk_slots_matrix.append([])
			for x in range(params.size.x):
				_chunk_slots_matrix[y].append([])
				for z in range(params.size.z):
					_chunk_slots_matrix[y][x].append(null)

		var start := position
		var end := position + map_params.chunk_size
		end.x = min(end.x, map_params.size.x)
		end.y = min(end.y, map_params.size.y)
		end.z = min(end.z, map_params.size.z)

		for y in range(start.y, end.y):
			for x in range(start.x, end.x):
				for z in range(start.z, end.z):
					var slot: WFCSlot.Slot = _all_slots_matrix[y][x][z]
					_chunk_slots_set[slot] = true
					_chunk_slots_matrix[y][x][z] = slot

	func within_chunk(point: Vector3) -> bool:
		return \
			(point.y >= position.y and point.y < position.y + map_params.chunk_size.y and point.y < map_params.size.y) and \
			(point.x >= position.x and point.x < position.x + map_params.chunk_size.x and point.x < map_params.size.x) and \
			(point.z >= position.z and point.z < position.z + map_params.chunk_size.z and point.z < map_params.size.z)

	func get_overlapping(other: MapChunk) -> Array:
		var result := []
		var others := other._chunk_slots_set
		for slot in _chunk_slots_set.keys():
			if slot in others:
				result.append(slot)
		return result

	func reset_overlapping(other: MapChunk):
		for slot in get_overlapping(other):
			slot.expand(WFC._proto_data.keys())
			WFC._slot_reset.call_deferred(slot.position, WFC._proto_data.keys())

	func propagate_from(other: MapChunk):
		for slot in other._chunk_slots_set.keys():
			if not _get_neighbors(slot.position).is_empty():
				_propagate(slot)

	func _apply_custom_constraints():
		# TODO - OPTIMIZE?
		# Only allowed protos on the bottom are sand and empty
		var constrained_to_bottom = []
		for proto in WFC._proto_data:
			if WFC._proto_data[proto]["constrain_to"] == "BOT":
				constrained_to_bottom.append(proto)

		var chunk_top_y = min(position.y + map_params.chunk_size.y, map_params.size.y) - 1

		# no "uncapped" prototypes along the sides of the space
		var queue := []
		for slot in _chunk_slots_set.keys():
			if slot.position.y == 0:
				if slot.constrain_uncapped(Vector3.MODEL_BOTTOM):
					queue.append(slot)
			elif slot.remove_all(constrained_to_bottom):
				queue.append(slot)

			if slot.position.y == chunk_top_y and slot.constrain_uncapped(Vector3.MODEL_TOP):
				queue.append(slot)
			if slot.position.x == 0 and slot.constrain_uncapped(Vector3.MODEL_RIGHT):
				queue.append(slot)
			if slot.position.x == map_params.size.x - 1 and slot.constrain_uncapped(Vector3.MODEL_LEFT):
				queue.append(slot)
			if slot.position.z == 0 and slot.constrain_uncapped(Vector3.MODEL_REAR):
				queue.append(slot)
			if slot.position.z == map_params.size.z - 1 and slot.constrain_uncapped(Vector3.MODEL_FRONT):
				queue.append(slot)

		for slot in queue:
			_propagate(slot)

	func _collapse_next() -> bool:
		var selected = _select_lowest_entropy()
		if selected == null:
			return true

		selected.collapse()
		WFC._slot_constrained.call_deferred(selected.position, selected.possibilities)
		_propagate(selected)
		return false
	
	func _apply_custom_entropy(entropy: int, slot_position: Vector3) -> int:
		if slot_position.y == 0:
			entropy += 2
		else:
			entropy += floor(slot_position.y)

		if slot_position.x == 0:
			entropy += 1
		if slot_position.x == map_params.size.x - 1:
			entropy += 1
		if slot_position.z == 0:
			entropy += 1
		if slot_position.z == map_params.size.z - 1:
			entropy += 1

		return entropy

	func _select_lowest_entropy() -> WFCSlot.Slot:
		var lowest_entropy_value = 99999
		var lowest_entropy_slots

		for slot in _chunk_slots_set.keys():
			var entropy = slot.entropy()
			if entropy <= 1 or entropy > lowest_entropy_value:
				continue

			entropy = _apply_custom_entropy(entropy, slot.position)
			if entropy < lowest_entropy_value:
				lowest_entropy_value = entropy
				lowest_entropy_slots = [slot]
			else:
				lowest_entropy_slots.append(slot)

		if not lowest_entropy_slots:
			return null

		return lowest_entropy_slots[randi() % len(lowest_entropy_slots)]

	func _get_neighbors(slot_position: Vector3):
		var all_neighbors = [
			slot_position + Vector3(1, 0, 0),
			slot_position + Vector3(-1, 0, 0),
			slot_position + Vector3(0, 1, 0),
			slot_position + Vector3(0, -1, 0),
			slot_position + Vector3(0, 0, 1),
			slot_position + Vector3(0, 0, -1)
		]

		var valid_neighbors = []
		for neighbor_position in all_neighbors:
			if not within_chunk(neighbor_position):
				continue

			var slot = _chunk_slots_matrix[neighbor_position.y][neighbor_position.x][neighbor_position.z]
			if slot: 
				valid_neighbors.append(slot)

		return valid_neighbors

	func _propagate(slot: WFCSlot.Slot):
		var incomplete = _propagate_recursive(slot)
		while len(incomplete) > 0:
			var current = incomplete.pop_front()
			var inner_incomplete = _propagate_recursive(current)
			if len(inner_incomplete) > 0:
				print("Warning! Maxed call stack at least twice! Adding ", len(inner_incomplete), " additional propagations to queue of length ", len(incomplete))
				incomplete.append_array(inner_incomplete)

	func _propagate_recursive(slot: WFCSlot.Slot, depth: int = 0):
		var incomplete = [] # Slots that should be propagated, but we can't without hitting recursion limit

		for neighbor in _get_neighbors(slot.position):
			if neighbor.is_collapsed():
				continue

			var new_neighbor_possibilities = []
			for neighbor_proto in neighbor.possibilities:
				for proto in slot.possibilities:
					var direction = slot.position.direction_to(neighbor.position)
					if neighbor_proto in WFC._valid_neighbors[proto][direction]:
						new_neighbor_possibilities.append(neighbor_proto)
						break

			if len(new_neighbor_possibilities) != len(neighbor.possibilities):
				if len(new_neighbor_possibilities) == 0:
					# print("overcollapsed!")
					# WFC.stop_collapse.call_deferred()
					break

				neighbor.constrain(new_neighbor_possibilities)
				WFC._slot_constrained.call_deferred(neighbor.position, neighbor.possibilities)

				if depth >= 1000:
					incomplete.append(neighbor)
				else:
					incomplete.append_array(_propagate_recursive(neighbor, depth + 1))

		return incomplete
