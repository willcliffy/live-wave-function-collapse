extends Node

signal slot_constrained(slot: Vector3, protos: Array)
signal slot_reset(slot: Vector3, protos: Array)

signal map_initialized
signal map_collapsed

const PROTO_FILE_NAME = "prototype_data.json"

var _proto_data: Dictionary
var _valid_neighbors: Dictionary

var _thread: Thread
var _collapser: Collapser.WfcCollapser

var _autocollapse := false
var _autocollapse_started: float


func _ready():
	_load_proto_data()

	_thread = Thread.new()
	_collapser = Collapser.WfcCollapser.new()
	_thread.start(_collapser.run)


func _process(_delta):
	if _autocollapse and _collapser.idle:
		var action := Collapser.Action.new()
		action.type = Collapser.ActionType.COLLAPSE
		_collapser.queue_action(action)


func _load_proto_data():
	const pX = 0
	const pY = 1
	const nX = 2
	const nY = 3
	const pZ = 4
	const nZ = 5

	if not FileAccess.file_exists(PROTO_FILE_NAME):
		print("File not found.")
		return

	var file = FileAccess.open(PROTO_FILE_NAME, FileAccess.READ)
	var json_text = file.get_as_text()
	file.close()

	_proto_data = JSON.parse_string(json_text)
	if typeof(_proto_data) != TYPE_DICTIONARY:
		print("Failed to parse JSON.")
		return

	_valid_neighbors = {}
	for proto in _proto_data:
		var proto_datum = _proto_data[proto]
		_valid_neighbors[proto] = {}
		_valid_neighbors[proto][Vector3.MODEL_TOP] = proto_datum["valid_neighbours"][pZ]
		_valid_neighbors[proto][Vector3.MODEL_BOTTOM] = proto_datum["valid_neighbours"][nZ]
		_valid_neighbors[proto][Vector3.MODEL_LEFT] = proto_datum["valid_neighbours"][pX]
		_valid_neighbors[proto][Vector3.MODEL_RIGHT] = proto_datum["valid_neighbours"][nX]
		_valid_neighbors[proto][Vector3.MODEL_FRONT] = proto_datum["valid_neighbours"][nY]
		_valid_neighbors[proto][Vector3.MODEL_REAR] = proto_datum["valid_neighbours"][pY]


func initialize(params: WFCModels.MapParams):
	_collapser.initialize(params)


func _map_initialized():
	map_initialized.emit()


func _slot_constrained(slot: Vector3, protos: Array):
	slot_constrained.emit(slot, protos)


func _slot_reset(slot: Vector3, protos: Array):
	slot_reset.emit(slot, protos)


func start_collapse():
	_autocollapse = true
	_autocollapse_started = Time.get_unix_time_from_system()
	print(Time.get_datetime_string_from_system(), " autocollapse starting")


func stop_collapse():
	_autocollapse = false
	var elapsed = Time.get_unix_time_from_system() - _autocollapse_started
	print(Time.get_datetime_string_from_system(), " autocollapse stopped. Elapsed: ", elapsed)
	map_collapsed.emit()

