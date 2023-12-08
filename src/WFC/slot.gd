extends Node

class_name WFCSlot


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

	func constrain_uncapped(direction: Vector3) -> bool:
		var new_possibilities = []
		for proto in possibilities:
			if "p-1" in WFC._valid_neighbors[proto][direction]:
				new_possibilities.append(proto)

		var changed := len(new_possibilities) != len(possibilities)
		if changed:
			possibilities = new_possibilities
		return changed

	func remove_all(to_remove: Array) -> bool:
		var new_possibilities = []
		for proto in possibilities:
			if not proto in to_remove:
				new_possibilities.append(proto)

		var changed := len(new_possibilities) != len(possibilities)
		if changed:
			possibilities = new_possibilities
		return changed

	func entropy() -> int:
		return len(possibilities)

	func is_collapsed() -> bool:
		return len(possibilities) <= 1

	func _choose_from_bucket():
		var sum_of_weights := 0.0
		for proto in possibilities:
			sum_of_weights += WFC._proto_data[proto]["weight"]

		var selected_weight := randf_range(0, sum_of_weights)
		for proto in possibilities:
			selected_weight -= WFC._proto_data[proto]["weight"]
			if selected_weight <= 0.0:
				return proto

		return possibilities.back()

	func adjacent_to(other: Slot) -> bool:
		return position.distance_to(other.position) == 1
