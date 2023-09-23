extends Node


const PROTO_FILE_NAME = "prototype_data.json"

const pX = 0
const pY = 1
const nX = 2
const nY = 3
const pZ = 4
const nZ = 5


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

	func constrain_from_propagation(changed: Superposition) -> bool:
		var new_superposition = []
		for proto_1 in changed.get_possibilities():
			for proto_2 in get_possibilities():
				if WFCUtils.protos_compatible(proto_1, changed.position, proto_2, position):
					new_superposition.append(proto_2)

		if len(superposition) != len(new_superposition):
			constrain(new_superposition)
			return true

		return false


class Propagation:
	var superposition: Superposition
	var visited: bool = false
	var neighbor_propagations: Array


class WaveFunction:
	signal superposition_collapsed(Superposition)
	signal superposition_constrained(Superposition)

	# Whether or not we should explicitly collapse a cell when there is no root propagation to process
	# Start with WFC paused to avoid throttling on startup
	var paused: bool = true

	# size of the WFC space. This version does not support infinite terrain (yet) 
	var size: Vector3

	# a y by x by z matrix of Superpositions
	# that is: superpositions[y][x][z] =  Superposition{Vector3(x, y, z), ["proto1", "proto2", ..., "protoZ"], ""}
	# Note that we put y first since Godot is y-up
	var superpositions: Array

	# Once a cell is explicitly collapsed, it begins a propagations step.
	# A propagation can spawn other propagation steps, which should be processed before finishing the root propagation step.
	var root_propagation: Propagation
	
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

	func process(_delta) -> void:
		if paused:
			return

		for i in range(100):
			if root_propagation:
				process_propagation(_delta)
			else:
				collapse_next()

	func collapse_next() -> void:
		# Select a random superposition from the list of lowest-entropy superpositions
		var lowest_entropy_superpositions = _get_lowest_entropy_superpositions()
		
		if len(lowest_entropy_superpositions) == 0:
			print("think we're done")
			paused = true
			return
		
		var superposition: Superposition = lowest_entropy_superpositions[randi() % len(lowest_entropy_superpositions)]
		
		# Collapse the lowest-entropy position's superposition
		var collapsed = superposition.collapse()
		if not collapsed:
			print("Failed to collapse superposition! ", superposition.get_possibilities())
			return

		root_propagation = Propagation.new()
		root_propagation.superposition = superposition
		superposition_collapsed.emit(superposition)

	func process_propagation(_delta):
		if not root_propagation.visited:
			# Root propagation is always an explicit collapse, so we don't need to constrain here
			root_propagation.neighbor_propagations = _get_neighboring_propagations(root_propagation.superposition.position)
			root_propagation.visited = true
			return

		if root_propagation.neighbor_propagations.is_empty():
			# We've generated neighboring propogations (and maybe traversed them) and now there are no other constrain steps
			# We are done.
			print("all neighbors propagated")
			root_propagation = null
			return

		# Go to the first unvisited propagation
		var last = root_propagation
		var current: Propagation = root_propagation.neighbor_propagations[0]

		while current.visited:
			if current.neighbor_propagations.is_empty():
				# If we've visited this node and it has no propagations, either:
				# - there were no propagations necessary
				# - propagations were necessary, and they were completed.
				# We can remove this node from it's parent
				last.neighbor_propagations.pop_front()
				return
			last = current
			current = current.neighbor_propagations[0]

		#print(last.superposition.position, " ", current.superposition.position)

		# We found a node that we haven't travelled to yet
		# Constrain it, and note which of its neighbors we need to visit next
		current.visited = true
		var constrained = current.superposition.constrain_from_propagation(root_propagation.superposition)
		if constrained:
			current.neighbor_propagations = _get_neighboring_propagations(current.superposition.position)
			superposition_constrained.emit(current.superposition)

	func _get_neighboring_propagations(position: Vector3) -> Array:
		var neighbor_propagations := []
		for superposition in _get_neighboring_superpositions(position):
			var neighbor_propagation = Propagation.new()
			neighbor_propagation.superposition = superposition
			neighbor_propagations.append(neighbor_propagation)

		return neighbor_propagations

	func _get_neighboring_superpositions(position: Vector3):
		var directions = []

		if position.x < size.x - 1:
			directions.append(position + Vector3(1, 0, 0))
		if position.x > 0:
			directions.append(position + Vector3(-1, 0, 0))

		if position.y < size.y - 1:
			directions.append(position + Vector3(0, 1, 0))
		if position.y > 0:
			directions.append(position + Vector3(0, -1, 0))

		if position.z < size.z - 1:
			directions.append(position + Vector3(0, 0, 1))
		if position.z > 0:
			directions.append(position + Vector3(0, 0, -1))

		var result = []
		for direction in directions:
			result.append(superpositions[direction.y][direction.x][direction.z])

		return result

	func _get_lowest_entropy_superpositions() -> Array:
		var lowest_entropy_value = INF
		var lowest_entropy_superpositions = []

		for x in range(size.x):
			for y in range(size.y):
				for z in range(size.z):
					var superposition = superpositions[y][x][z]
					var entropy = superposition.get_entropy()
					if entropy > 1 and entropy < lowest_entropy_value:
						lowest_entropy_value = entropy
						lowest_entropy_superpositions = [superposition]
					elif entropy == lowest_entropy_value:
						lowest_entropy_superpositions.append(superposition)

		return lowest_entropy_superpositions
