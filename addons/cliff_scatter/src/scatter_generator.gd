@tool
extends MultiMeshInstance3D

signal rebuild()

@export_category("CliffScatterGenerator")

@export_group("General")

@export
var scatter_mesh: Mesh

@export var base_placement_noise: FastNoiseLite:
	set(val):
		base_placement_noise = val
		if _is_ready: _build_queued = true

@export var base_placement_noise_threshold: float = .25:
	set(val):
		base_placement_noise_threshold = val
		if _is_ready: _build_queued = true

@export var shape_size: Vector3 = Vector3(10, 10, 10):
	set(val):
		shape_size = val
		if _is_ready: _build_queued = true

@export_range(0, 100, 2)
var samples_per_meter: int = 8:
	set(val):
		samples_per_meter = val
		if _is_ready: _build_queued = true

@export
var disabled: bool = true


var _build_queued := false
var _scanning := false
var _is_ready := false

const RAYCASTS_PER_FRAME = 50
var scan_queue := []

var space_state: PhysicsDirectSpaceState3D
var current_scan: ScatterModels.TerrainScan

# offset the xz coordinates slightly for raycasts to avoid getting stuck in gaps between modules
# Maybe one day combine the modules into one mesh and recreate the collision shape, but until then this is fine
const RAYCAST_XZ_OFFSET := 0.001


func _ready():
	if disabled:
		return
	space_state = get_world_3d().get_direct_space_state()
	rebuild.connect(func(): _build_queued = true)
	_is_ready = true


func _physics_process(delta):
	if not _is_ready:
		return

	if _scanning:
		var done := _process_scan()
		if done:
			_scanning = false
			print(Time.get_datetime_string_from_system(), " terrain scan complete")
			ResourceSaver.save(current_scan.height_map.image, "res://_game/terrain_scans/height_map.tres")
			ResourceSaver.save(current_scan.normal_map.image, "res://_game/terrain_scans/normal_map.tres")
			var features := _build_features()
			for feature in features:
				place_feature(feature)
	elif _build_queued:
		_build_queued = false
		_scanning = true
		_begin_terrain_scan()


func _build_features() -> Array:
	# populate placement, scale, and rotation maps
	var grass_feature_input = ScatterModels.ScatterFeatureInput.new()
	grass_feature_input.mesh = scatter_mesh
	grass_feature_input.sample_resolution = samples_per_meter
	grass_feature_input.placement_noise = base_placement_noise
	grass_feature_input.placement_noise_threshold = base_placement_noise_threshold

	var grass_feature = ScatterModels.ScatterFeature.new()
	grass_feature.initialize(current_scan, grass_feature_input)

	ResourceSaver.save(grass_feature.placement_map.image, "res://_game/terrain_scans/placement_map.tres")
	ResourceSaver.save(grass_feature.rotation_map.image, "res://_game/terrain_scans/rotation_map.tres")
	ResourceSaver.save(grass_feature.scale_map.image, "res://_game/terrain_scans/scale_map.tres")

	return [grass_feature]


func _begin_terrain_scan():
	print(Time.get_datetime_string_from_system(), " starting terrain scan")

	current_scan = ScatterModels.TerrainScan.new()
	current_scan.initialize(position, shape_size, samples_per_meter)

	current_scan.height_map = ScatterModels.WorldSpaceMapRGB.new()
	current_scan.height_map.initialize(current_scan)
	current_scan.normal_map = ScatterModels.WorldSpaceMapRGB.new()
	current_scan.normal_map.initialize(current_scan)

	var x_world_start := position.x
	var x_world_end := position.x + shape_size.x
	var z_world_start := position.z
	var z_world_end := position.z + shape_size.z

	var step := 1.0 / float(samples_per_meter) # in meters

	var x_world := x_world_start
	while x_world < x_world_end:
		var z_world := z_world_start
		while z_world < z_world_end:
			scan_queue.append([x_world, z_world])
			z_world += step
		x_world += step


func _process_scan() -> bool:
	for i in range(RAYCASTS_PER_FRAME):
		if len(scan_queue) == 0:
			return true

		var current_coords := scan_queue.pop_front()
		var x_world = current_coords[0]
		var z_world = current_coords[1]

		var cast_origin := Vector3(
			x_world + RAYCAST_XZ_OFFSET,
			shape_size.y + 1.0,
			z_world + RAYCAST_XZ_OFFSET
		)
		var cast_end := cast_origin + Vector3.DOWN * (shape_size.y + 2.0)
		var query := PhysicsRayQueryParameters3D.create(cast_origin, cast_end)
		query.collide_with_areas = true

		var raycast_result := space_state.intersect_ray(query)
		if raycast_result != null and "position" in raycast_result and "normal" in raycast_result:
			current_scan.height_map.set_value(x_world, z_world, raycast_result["position"])
			current_scan.normal_map.set_value(x_world, z_world, raycast_result["normal"])

	return len(scan_queue) == 0


func _process_place_feature():
	pass


func place_feature(feature: ScatterModels.ScatterFeature):
	var instances = []
	var image_resolution := feature.terrain.height_map.image.get_size()
	for x in image_resolution.x:
		for y in image_resolution.y:
			var placement := feature.placement_map._get_pixel(x, y)
			if placement.is_equal_approx(Vector3.ZERO):
				continue

			var instance_position := feature.terrain.height_map._get_pixel(x, y) - position
			if instance_position.is_equal_approx(Vector3.ZERO):
				continue

			var instance_transform := Transform3D(Basis(), instance_position)

			var instance_scale := feature.scale_map._get_pixel(x, y)
			instance_transform = instance_transform.scaled_local(instance_scale)

			var instance_rotation := feature.rotation_map._get_pixel(x, y)
			instance_transform = instance_transform \
				.rotated_local(Vector3(1, 0, 0), instance_rotation.x) \
				.rotated_local(Vector3(0, 1, 0), instance_rotation.y) \
				.rotated_local(Vector3(0, 0, 1), instance_rotation.z)

			instances.append(instance_transform)

	multimesh = MultiMesh.new()
	multimesh.mesh = scatter_mesh
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.instance_count = len(instances)

	for i in range(len(instances)):
		multimesh.set_instance_transform(i, instances[i])

	print(Time.get_datetime_string_from_system(), " Spawned ", len(instances), " instances of grass")

