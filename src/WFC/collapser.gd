extends Node

class_name Collapser


const AUTOCOLLAPSE_SPEED = 5


enum ActionType {
	INITIALIZE = 1,
	COLLAPSE = 2,
}


class Action:
	var type: ActionType
	var data: Variant


class WfcCollapser:
	# thread i/o
	var idle := false
	var _stop := false
	var _runner := Semaphore.new()
	var _queued_actions := []

	var params: WFCModels.MapParams

	var all_slots_matrix := []
	var all_slots_array := []

	var all_chunks_array := []
	var current_chunk_index: int = 0

	func initialize(input_params: WFCModels.MapParams):
		params = input_params

		var action := Action.new()
		action.type = ActionType.INITIALIZE
		queue_action(action)

	func queue_action(action: Action):
		_queued_actions.push_front(action)
		_runner.post()

	func run():
		while true:
			idle = true
			_runner.wait()
			idle = false

			if _stop: break

			if len(_queued_actions) <= 0:
				print("Posted but no action queued!")
				continue

			var action = _queued_actions.pop_back()
			if action.type == ActionType.INITIALIZE:
				_generate_slots()
				_generate_chunks()
				all_chunks_array[current_chunk_index]._apply_custom_constraints()
				WFC._map_initialized.call_deferred()
			elif action.type == ActionType.COLLAPSE:
				#print("skipping collapse cause shit broke")
				#_stop = true
				for i in range(AUTOCOLLAPSE_SPEED):
					var current_chunk = all_chunks_array[current_chunk_index]
					var done = current_chunk._collapse_next()
					if done:
						current_chunk_index += 1
						if current_chunk_index >= len(all_chunks_array):
							print("done")
							WFC.stop_collapse.call_deferred()
						else:
							var next_chunk = all_chunks_array[current_chunk_index]
							for j in range(current_chunk_index):
								next_chunk.reset_overlapping(all_chunks_array[j])
								next_chunk.propagate_from(all_chunks_array[j])
							next_chunk._apply_custom_constraints()
						break
			else:
				print("Invalid action queued, skipping: ", action.type)

	func stop():
		_stop = true
		_runner.post()

	func _generate_slots():
		for y in range(params.size.y):
			all_slots_matrix.append([])
			for x in range(params.size.x):
				all_slots_matrix[y].append([])
				for z in range(params.size.z):
					var slot = WFCChunk.Slot.new()
					slot.position = Vector3(x, y, z)
					slot.expand(WFC._proto_data.keys())
					all_slots_array.append(slot)
					all_slots_matrix[y][x].append(slot)

	func _generate_chunks():
		var num_x = floor(params.size.x / (params.chunk_size.x - params.chunk_overlap)) + 1
		var num_y = floor(params.size.y / (params.chunk_size.y - params.chunk_overlap)) + 1
		var num_z = floor(params.size.z / (params.chunk_size.z - params.chunk_overlap)) + 1
		var position_factor := params.chunk_size - Vector3.ONE * params.chunk_overlap

		for x_chunk in range(num_x):
			for y_chunk in range(num_y):
				for z_chunk in range(num_z):
					var position := position_factor * Vector3(x_chunk, y_chunk, z_chunk)
					var new_chunk := WFCChunk.MapChunk.new()
					new_chunk.initialize(params, position, all_slots_matrix, all_slots_array)
					all_chunks_array.append(new_chunk)
