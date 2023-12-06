extends Node

class_name WFCChunk


class Slot:
	var position: Vector3
	var possibilities: Array

	func expand(protos: Array):
		possibilities = protos

	func constrain(protos: Array):
		possibilities = protos

	func collapse(proto: String = ""):
		if not proto.is_empty():
			possibilities = [proto]
		else:
			possibilities = [_choose_from_bucket()]

	func constrain_uncapped(direction: Vector3):
		var new_possibilities = []
		for proto in possibilities:
			if "p-1" in WFC._valid_neighbors[proto][direction]:
				new_possibilities.append(proto)
				new_possibilities.append(proto)

		if len(new_possibilities) != len(possibilities):
			possibilities = new_possibilities

	func remove_all(to_remove: Array):
		var new_possibilities = []
		for proto in possibilities:
			if not proto in to_remove:
				new_possibilities.append(proto)
		if len(new_possibilities) != len(possibilities):
			possibilities = new_possibilities

	func entropy() -> int:
		return len(possibilities)

	func is_collapsed() -> bool:
		return len(possibilities) <= 1

	func _choose_from_bucket():
		var sum_of_weights := 0
		for proto in possibilities:
			sum_of_weights += WFC._proto_data[proto]["weight"]

		var selected_weight := randi_range(0, sum_of_weights)
		for proto in possibilities:
			selected_weight -= WFC._proto_data[proto]["weight"]
			if selected_weight <= 0:
				return proto

		return possibilities.back()


class MapChunk:
	var map_params: WFCModels.MapParams
	var position: Vector3

	var _slots_array: Array = []
	var _slots_matrix: Array = []

	func initialize(params: WFCModels.MapParams, chunk_position: Vector3, slots_matrix: Array, slots_array: Array):
		map_params = params
		position = chunk_position
		_slots_array = slots_array
		_slots_matrix = slots_matrix

	func within_chunk(point: Vector3) -> bool:
		return \
			(point.x >= position.x and point.x < position.x + map_params.chunk_size.x and point.x < map_params.size.x) and \
			(point.y >= position.y and point.y < position.y + map_params.chunk_size.y and point.y < map_params.size.y) and \
			(point.z >= position.z and point.z < position.z + map_params.chunk_size.z and point.z < map_params.size.z)

	func get_slots_within_chunk() -> Array:
		# TODO - OPTIMIZE
		var result := []
		for slot in _slots_array:
			if within_chunk(slot.position):
				result.append(slot)
		return result

	func get_overlapping(other: MapChunk) -> Array:
		# TODO - OPTIMIZE
		var result := []
		var others := other.get_slots_within_chunk()
		for slot in get_slots_within_chunk():
			if slot in others:
				result.append(slot)
		return result

	func reset_overlapping(other: MapChunk):
		# TODO - OPTIMIZE
		for slot in get_overlapping(other):
			slot.expand(WFC._proto_data.keys())
			WFC._slot_reset.call_deferred(slot.position, slot.possibilities)

	func propagate_from(other: MapChunk):
		# TODO - OPTIMIZE
		for slot in other.get_slots_within_chunk():
			_propagate(slot)

	func _apply_custom_constraints():
		# TODO - OPTIMIZE?
		# Only allowed protos on the bottom are sand and empty
		var constrained_to_bottom = []
		for proto in WFC._proto_data:
			if WFC._proto_data[proto]["constrain_to"] == "BOT":
				constrained_to_bottom.append(proto)

		# no "uncapped" prototypes along the sides of the space
		var propagation_queue := []
		for y in range(map_params.size.y):
			for x in range(map_params.size.x):
				for z in range(map_params.size.z):
					var slot = _slots_matrix[y][x][z]
					if not slot:
						continue

					if y == 0:
						slot.constrain_uncapped(Vector3.MODEL_BOTTOM)
						propagation_queue.append(slot)
					else:
						slot.remove_all(constrained_to_bottom)
						propagation_queue.append(slot)

					if y == map_params.size.y - 1:
						slot.constrain_uncapped(Vector3.MODEL_TOP)
						propagation_queue.append(slot)
					if x == 0:
						slot.constrain_uncapped(Vector3.MODEL_RIGHT)
						propagation_queue.append(slot)
					if x == map_params.size.x - 1:
						slot.constrain_uncapped(Vector3.MODEL_LEFT)
						propagation_queue.append(slot)
					if z == 0:
						slot.constrain_uncapped(Vector3.MODEL_REAR)
						propagation_queue.append(slot)
					if z == map_params.size.z - 1:
						slot.constrain_uncapped(Vector3.MODEL_FRONT)
						propagation_queue.append(slot)

		for propagation in propagation_queue:
			_propagate(propagation)

	func _collapse_next() -> bool:
		var selected = _select_lowest_entropy()
		if selected == null:
			return true

		selected.collapse()
		WFC._slot_constrained.call_deferred(selected.position, selected.possibilities)
		_propagate(selected)
		return false

	func _select_lowest_entropy() -> Slot:
		var lowest_entropy_value = 99999
		var lowest_entropy_slots

		for slot in get_slots_within_chunk():
			var entropy = slot.entropy()
			if entropy <= 1 or entropy > lowest_entropy_value:
				continue

			entropy += slot.position.y
			if slot.position.y == map_params.size.y:
				entropy += 2
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
		for neighbor in all_neighbors:
			if not within_chunk(neighbor):
				continue
			var slot = _slots_matrix[neighbor.y][neighbor.x][neighbor.z]
			if slot:
				valid_neighbors.append(slot)

		return valid_neighbors

	func _propagate(slot: Slot):
		var incomplete = _propagate_recursive(slot)
		while len(incomplete) > 0:
			var current = incomplete.pop_front()
			var inner_incomplete = _propagate_recursive(current)
			if len(inner_incomplete) > 0:
				print("Warning! Maxed call stack at least twice! Adding ", len(inner_incomplete), " additional propagations to queue of length ", len(incomplete))
				incomplete.append_array(inner_incomplete)

	func _propagate_recursive(slot: Slot, depth: int = 0):
		var incomplete = [] # Slots that should be propagated, but we can't without hitting recursion limit
		for neighbor in _get_neighbors(slot.position):
			if neighbor.is_collapsed(): continue
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
