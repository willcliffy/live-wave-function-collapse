extends Node


const PROTO_FILE_NAME = "prototype_data.json"

const pX = 0
const pY = 1
const nX = 2
const nY = 3
const pZ = 4
const nZ = 5

const time_before_next_tick_msec = 1.0 / 60.0 * 1000 * 0.75


class WFCUtils:
	static var loaded: bool = false
	static var proto_data: Dictionary
	
	static func load_data():
		if loaded:
			print("Tried to double load!")
			return

		if not FileAccess.file_exists(PROTO_FILE_NAME):
			print("File not found.")
			return

		var file = FileAccess.open(PROTO_FILE_NAME, FileAccess.READ)
		var json_text = file.get_as_text()
		file.close()

		proto_data = JSON.parse_string(json_text)
		if typeof(proto_data) != TYPE_DICTIONARY:
			print("Failed to parse JSON.")
			return

	static func protos_compatible(
		proto_1: String,
		proto_1_position: Vector3,
		proto_2: String,
		proto_2_position: Vector3
	) -> bool:
		var direction_ind = _direction_index(proto_1_position, proto_2_position)
		return proto_2 in proto_data[proto_1]["valid_neighbours"][direction_ind]

	static func proto_uncapped(proto: String, direction: Vector3 = Vector3.UP) -> bool:
		var direction_ind = _direction_index(Vector3.ZERO, direction)
		return not "p-1" in proto_data[proto]["valid_neighbours"][direction_ind]

	static func get_uncapped_protos(direction: Vector3 = Vector3.UP) -> Array:
		var protos = []
		for proto_name in proto_data:
			if proto_uncapped(proto_name, direction):
				continue
			protos.append(proto_name)
		return protos

	static func _direction_index(from: Vector3, to: Vector3, tolerance: float = 0.1) -> int:
		## NOTE that this function swaps Z and Y because of the shape of the proto data vs godot being Y-up
		if to.x > from.x + tolerance:
			return pX
		elif to.x < from.x - tolerance:
			return nX

		if to.y > from.y + tolerance:
			return pZ
		elif to.y < from.y - tolerance:
			return nZ

		if to.z > from.z + tolerance:
			return nY
		elif to.z < from.z - tolerance:
			return pY

		print(from, " ", to)
		return -1


class Superposition:
	var position: Vector3
	var superposition: Array
	var collapsed_to: String
	
	func is_collapsed():
		return not collapsed_to.is_empty()
	
	func get_entropy():
		return 1 if is_collapsed() else len(superposition)
	
	func get_possibilities():
		return [collapsed_to] if is_collapsed() else superposition

	func constrain(new_superposition: Array) -> bool:
		if len(new_superposition) >= len(superposition):
			return false

		superposition = new_superposition
		if len(superposition) == 1:
			collapsed_to = superposition[0]

		return true

	func collapse(proto: String = String()) -> bool:
		if is_collapsed():
			return false

		if len(superposition) == 0:
			return false

		if proto.is_empty():
			proto = superposition[randi() % len(superposition)]

		collapsed_to = proto

		return true

	func constrained_from_propagation(changed: Superposition) -> Array:
		var new_superposition = []
		for proto_1 in changed.get_possibilities():
			for proto_2 in get_possibilities():
				if proto_2 in new_superposition:
					continue
				if WFCUtils.protos_compatible(proto_1, changed.position, proto_2, position):
					new_superposition.append(proto_2)

		return new_superposition


class Propagation:
	# The superposition which was constrained or collapsed to begin this propagation
	var superposition: Superposition
	
	# Whether or not the neighbors field has been populated
	var visited: bool = false
	
	# Whether or not the neighbors field has been checked for changes
	var processed: bool = false
	
	# if this propagation is in another propagation's neighbor list, 
	var parent: Propagation

	# The neighboring superpositions which may or may not be impacted by the change in superposition
	var neighbors: Array

	func get_unprocessed_child() -> Propagation:
		if visited and not processed:
			return self

		for neighbor in neighbors:
			var child = neighbor.get_unprocessed_child()
			if child != null:
				return child

		return null

	func get_unvisited_child() -> Propagation:
		if not visited:
			return self

		for neighbor in neighbors:
			var child = neighbor.get_unvisited_child()
			if child != null:
				return child

		return null

	func all_positions() -> Array:
		var positions = [superposition.position]
		for neighbor in neighbors:
			positions.append_array(neighbor.all_positions())
		return positions
	
	func get_root() -> Propagation:
		if parent == null:
			return self
		return parent.get_root()


class WaveFunction:
	signal superposition_collapsed(Superposition)
	signal superposition_constrained(Superposition)
	signal superposition_overcollapsed(position: Vector3)

	var autocollapsing: bool = true

	# size of the WFC space. This version does not support infinite terrain (yet) 
	var size: Vector3

	# a y by x by z matrix of Superpositions
	# that is: superpositions[y][x][z] =  Superposition{Vector3(x, y, z), ["proto1", "proto2", ..., "protoZ"], ""}
	# Note that we put y first since Godot is y-up
	var superpositions: Array

	# Once a cell is explicitly collapsed, it begins a propagations step.
	# A propagation can spawn other propagation steps, which should be processed before finishing the root propagation step.
	var root_propagations: Array
	var visited_positions: Array

	func initialize(input_size: Vector3):
		size = input_size

		WFCUtils.load_data()

		for y in range(size.y):
			superpositions.append([])
			for x in range(size.x):
				superpositions[y].append([])
				for z in range(size.z):
					var superposition = Superposition.new()
					superposition.position = Vector3(x, y, z)
					superposition.superposition = WFCUtils.proto_data.keys()
					superpositions[y][x].append(superposition)

	func apply_custom_constraints():
		# no "uncapped" prototypes along the top and sides of the space
		for y in range(size.y):
			for x in range(size.x):
				for z in range(size.z):
					var superposition: Superposition = superpositions[y][x][z]
					if y == size.y - 1:
						superposition.constrain(WFCUtils.get_uncapped_protos())
						root_propagations.push_front(new_propagation(superposition))
					if x == 0:
						superposition.constrain(WFCUtils.get_uncapped_protos(Vector3.MODEL_RIGHT))
						root_propagations.push_front(new_propagation(superposition))
					if x == size.x - 1:
						superposition.constrain(WFCUtils.get_uncapped_protos(Vector3.LEFT))
						root_propagations.push_front(new_propagation(superposition))
					if z == 0:
						superposition.constrain(WFCUtils.get_uncapped_protos(Vector3.MODEL_REAR))
						root_propagations.push_front(new_propagation(superposition))
					if z == size.z - 1:
						superposition.constrain(WFCUtils.get_uncapped_protos(Vector3.MODEL_FRONT))
						root_propagations.push_front(new_propagation(superposition))

	func process(delta) -> void:
		var s = Time.get_ticks_msec()
		while Time.get_ticks_msec() - s < time_before_next_tick_msec:
			if not root_propagations.is_empty():
				var propagation: Propagation = root_propagations.front()
				var unvisited = propagation.get_unvisited_child()
				if unvisited == null:
					print("Unvisited is null! ", propagation.get_unprocessed_child())
					continue

				visit_propagation(unvisited)
				process_visited_propagation(unvisited)
				break
			elif autocollapsing:
				collapse_next()
				break

	func collapse(position: Vector3, proto: String) -> void:
		if not root_propagations.is_empty():
			print("Tried to collapse but root propagations was not empty!")
			return

		var superposition = superpositions[position.y][position.x][position.z]
		var collapsed = superposition.collapse(proto)
		if not collapsed:
			print("Failed to collapse superposition! ", superposition.get_possibilities())
			return

		root_propagations = [new_propagation(superposition)]

	func collapse_next() -> void:
		if not root_propagations.is_empty():
			print("Tried to collapse but root propagations was not empty!")
			return

		# Get a list of the list of lowest-entropy superpositions
		var lowest_entropy_superpositions = _get_lowest_entropy_superpositions()
		if len(lowest_entropy_superpositions) == 0:
			print("Done!")
			autocollapsing = false
			return

		# Select a random superposition from the list
		var superposition: Superposition = lowest_entropy_superpositions[randi() % len(lowest_entropy_superpositions)]
		
		# Collapse the selected superposition
		var collapsed = superposition.collapse()
		if not collapsed:
			print("Failed to collapse superposition! ", superposition.get_possibilities())
			return

		# Propagate the change to neighboring cells.
		root_propagations = [new_propagation(superposition)]

#	func process_propagation_DEPRECATED(_delta):
#		var root_propagation = root_propagations[0] 
#		if not root_propagation.visited:
#			root_propagation.neighbor_propagations = _get_neighboring_propagations(root_propagation.superposition.position)
#			root_propagation.visited = true
#			return
#
#		if root_propagation.neighbor_propagations.is_empty():
#			#superposition_collapsed_release.emit(root_propagation.superposition)
#			root_propagations.pop_front()
#			visited_positions = []
#			return
#
#		# Go to the first unvisited propagation
#		var last = root_propagation
#		var current: Propagation = root_propagation.neighbor_propagations[0]
#		while current.visited:
#			if current.neighbor_propagations.is_empty():
#				superposition_constrained_release.emit(current.superposition)
#				last.neighbor_propagations.pop_front()
#				return
#			last = current
#			current = current.neighbor_propagations[0]
#
#		current.visited = true
#		visited_positions.append(current.superposition.position)
#		var new_superposition = current.superposition.constrained_from_propagation(last.superposition)
#		if len(new_superposition) != len(current.superposition.get_possibilities()):
#			current.superposition.constrain(new_superposition)
#			if len(new_superposition) == 0:
#				superposition_overcollapsed.emit(current.superposition.position)
#				return
#			if len(new_superposition) == 1:
#				visited_positions = []
#				var leaf_propagation = Propagation.new()
#				leaf_propagation.superposition = current.superposition
#				root_propagations.push_front(leaf_propagation)
#				superposition_collapsed.emit(current.superposition)
#				visited_positions = []
#			current.neighbor_propagations = _get_neighboring_propagations(current.superposition.position, visited_positions)
#			superposition_constrained.emit(current.superposition)

	func new_propagation(superposition: Superposition, parent: Propagation = null) -> Propagation:
		var propagation := Propagation.new()
		propagation.superposition = superposition
		propagation.parent = parent
		return propagation

	func visit_propagation(propagation: Propagation) -> void:
		var position := propagation.superposition.position
		var all_positions = propagation.get_root().all_positions()
		print(len(all_positions))

		var p_x_pos = position + Vector3(1, 0, 0)
		if position.x < size.x - 1 and not p_x_pos in all_positions:
			var neighbor_superposition = _get_superposition(p_x_pos)
			var new_propagation = new_propagation(neighbor_superposition, propagation)
			propagation.neighbors.append(new_propagation)

		var n_x_pos = position + Vector3(-1, 0, 0)
		if position.x > 0 and not n_x_pos in all_positions:
			var neighbor_superposition = _get_superposition(n_x_pos)
			var new_propagation = new_propagation(neighbor_superposition, propagation)
			propagation.neighbors.append(new_propagation)

		var p_y_pos = position + Vector3(0, 1, 0)
		if position.y < size.y - 1 and not p_y_pos in all_positions:
			var neighbor_superposition = _get_superposition(p_y_pos)
			var new_propagation = new_propagation(neighbor_superposition, propagation)
			propagation.neighbors.append(new_propagation)

		var n_y_pos = position + Vector3(0, -1, 0)
		if position.y > 0 and not n_y_pos in all_positions:
			var neighbor_superposition = _get_superposition(n_y_pos)
			var new_propagation = new_propagation(neighbor_superposition, propagation)
			propagation.neighbors.append(new_propagation)

		var p_z_pos = position + Vector3(0, 0, 1)
		if position.z < size.z - 1 and not p_z_pos in all_positions:
			var neighbor_superposition = _get_superposition(p_z_pos)
			var new_propagation = new_propagation(neighbor_superposition, propagation)
			propagation.neighbors.append(new_propagation)

		var n_z_pos = position + Vector3(0, 0, -1)
		if position.z > 0 and not n_z_pos in all_positions:
			var neighbor_superposition = _get_superposition(n_z_pos)
			var new_propagation = new_propagation(neighbor_superposition, propagation)
			propagation.neighbors.append(new_propagation)

		propagation.visited = true
		print("visited ", propagation.superposition.position, ", has neighbors ", len(propagation.neighbors))

	func process_visited_propagation(propagation: Propagation):
		for neighbor in propagation.neighbors:
			var new_superposition = neighbor.superposition.constrained_from_propagation(propagation.superposition)
			if len(new_superposition) == len(neighbor.superposition.superposition):
				propagation.neighbors.erase(new_superposition)
				continue

			neighbor.superposition.constrain(new_superposition)

			if len(new_superposition) == 0:
				autocollapsing = false
				superposition_overcollapsed.emit(neighbor.superposition)
			elif len(new_superposition) == 1:
				superposition_collapsed.emit(neighbor.superposition)
			else:
				superposition_constrained.emit(neighbor.superposition)

		propagation.processed = true
		print("processed ", propagation.superposition.position, " now has neighbors ", len(propagation.neighbors))

#	func _get_neighboring_propagations(position: Vector3, omit: Array = []) -> Array:
#		var neighbor_propagations := []
#		for superposition in _get_neighboring_superpositions(position, omit):
#			var neighbor_propagation = Propagation.new()
#			neighbor_propagation.superposition = superposition
#			neighbor_propagations.append(neighbor_propagation)
#
#		return neighbor_propagations
#
#	func _get_neighboring_superpositions(position: Vector3, omit: Array = []):
#		var directions = []
#
#		if position.x < size.x - 1:
#			directions.append(position + Vector3(1, 0, 0))
#		if position.x > 0:
#			directions.append(position + Vector3(-1, 0, 0))
#
#		if position.y < size.y - 1:
#			directions.append(position + Vector3(0, 1, 0))
#		if position.y > 0:
#			directions.append(position + Vector3(0, -1, 0))
#
#		if position.z < size.z - 1:
#			directions.append(position + Vector3(0, 0, 1))
#		if position.z > 0:
#			directions.append(position + Vector3(0, 0, -1))
#
#		var result = []
#		for direction in directions:
#			if direction in omit:
#				continue
#			var superposition = superpositions[direction.y][direction.x][direction.z]
#			if not superposition.is_collapsed():
#				result.append(superposition)
#
#		return result

	func _get_lowest_entropy_superpositions() -> Array:
		var lowest_entropy_value = INF
		var lowest_entropy_superpositions = []

		for x in range(size.x):
			for y in range(size.y):
				for z in range(size.z):
					var superposition = superpositions[y][x][z]
					var entropy = superposition.get_entropy()
					if y == size.y - 1:
						entropy += 2
					if y == size.y - 2:
						entropy += 1
					if entropy > 1 and entropy < lowest_entropy_value:
						lowest_entropy_value = entropy
						lowest_entropy_superpositions = [superposition]
					elif entropy == lowest_entropy_value:
						lowest_entropy_superpositions.append(superposition)

		return lowest_entropy_superpositions

	func _get_superposition(position: Vector3) -> Superposition:
		return superpositions[position.y][position.x][position.z]

