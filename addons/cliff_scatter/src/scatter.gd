@tool
extends MultiMeshInstance3D

signal rebuild()

const Models = preload("./models.gd")

@export_category("CliffScatter")

@export_group("General")

@export var base_noise: ImageTexture:
	set(val):
		base_noise = val
		if _is_ready: _build_queued = true

@export var base_noise_threshold: float = .25:
	set(val):
		base_noise_threshold = val
		if _is_ready: _build_queued = true

@export var shape_size: Vector3 = Vector3(10, 10, 10):
	set(val):
		shape_size = val
		if _is_ready: _build_queued = true


@export_group("Scatter Options")

@export_range(0.0, 360)
var max_rotation_degrees: float = 360:
	set(val):
		max_rotation_degrees = val
		max_rotation_radians = deg_to_rad(val)

var max_rotation_radians: float = 2 * PI:
	set(val):
		max_rotation_radians = val
		if _is_ready: _build_queued = true

@export_range(0.0, 180)
var max_tilt_degrees: float = 20:
	set(val):
		max_tilt_degrees = val
		max_tilt_radians = deg_to_rad(val)

var max_tilt_radians: float = PI / 8.0:
	set(val):
		max_tilt_radians = val
		if _is_ready: _build_queued = true

@export_range(0.0, 10.0)
var object_scale: float = 1:
	set(val):
		object_scale = val
		if _is_ready: _build_queued = true

@export_range(0.0, 1.0)
var max_scale_change: float = 0.15:
	set(val):
		max_scale_change = val
		if _is_ready: _build_queued = true

@export_range(0.0, 100.0)
var tile_size: float = 1.0:
	set(val):
		tile_size = val
		if _is_ready: _build_queued = true

@export_range(0, 100, 2)
var samples_per_meter: int = 8:
	set(val):
		samples_per_meter = val
		if _is_ready: _build_queued = true

@export_range(0, 360)
var max_angle_degrees: float = 60:
	set(val):
		max_angle_degrees = val
		max_angle_radians = deg_to_rad(val)

var max_angle_radians: float = PI / 4.0:
	set(val):
		max_angle_radians = val
		if _is_ready: _build_queued = true


@export_group("Cliff detection")

@export var detect_cliffs: bool  = true:
	set(val):
		detect_cliffs = val
		if _is_ready: _build_queued = true

@export
var unused_cliff_detection_skirt_meters: float:
	set(val):
		unused_cliff_detection_skirt_meters = val

@export_range(0, 10)
var cliff_detection_skirt_samples: int = 3:
	set(val):
		cliff_detection_skirt_samples = val
		unused_cliff_detection_skirt_meters = float(val) / float(samples_per_meter)
		if _is_ready: _build_queued = true

@export var cliff_detection_threshold:  float = 0.35:
	set(val):
		cliff_detection_threshold = val
		if _is_ready: _build_queued = true

#@export_group("Features")
#
#@export
#var features: Array = []

var _build_queued = false
var _is_ready = false


func _ready():
	rebuild.connect(func(): _build_queued = true)
	_is_ready = true


func _physics_process(delta):
	if _ready and _build_queued:
		_build_queued = false
		_rebuild()


func _rebuild():
	var scan_result := _scan_terrain()
	ResourceSaver.save(scan_result.height_map.image, "res://_game/heightmap.tres")
	ResourceSaver.save(scan_result.normal_map.image, "res://_game/normalmap.tres")
	var placement_map := create_base_placement_map(scan_result)
	ResourceSaver.save(placement_map, "res://_game/placementmap.tres")
	spawn_grass(placement_map, scan_result)


func _scan_terrain() -> Models.TerrainScan:
	print("scanning terrain")
	var space_state = get_world_3d().get_direct_space_state()

	# offset the xz coordinates slightly for raycasts to avoid getting stuck in gaps between modules
	# Maybe one day combine the modules into one mesh and recreate the collision shape, but until then this is fine
	const raycast_xz_offset = 0.001

	var height_map = Models.WorldSpaceMapRGB.new()
	height_map.initialize(position, shape_size, samples_per_meter)

	var normal_map = Models.WorldSpaceMapRGB.new()
	normal_map.initialize(position, shape_size, samples_per_meter)
	
	var x_world_start = position.x
	var x_world_end = position.x + shape_size.x - 1
	var z_world_start = position.z
	var z_world_end = position.z + shape_size.z - 1
	
	var step = 1.0 / float(samples_per_meter) # in meters
	
	var x_world = x_world_start
	while x_world < x_world_end:
		var z_world = z_world_start
		while z_world < z_world_end:
			var cast_origin = Vector3(
				x_world + raycast_xz_offset,
				shape_size.y + 1.0,
				z_world + raycast_xz_offset
			)
			var cast_end = cast_origin + Vector3.DOWN * (shape_size.y + 2.0)
			var query = PhysicsRayQueryParameters3D.create(cast_origin, cast_end)
			query.collide_with_areas = true

			var raycast_result = space_state.intersect_ray(query)
			if raycast_result != null and "position" in raycast_result and "normal" in raycast_result:
				height_map.set_value(x_world, z_world, raycast_result["position"])
				normal_map.set_value(x_world, z_world, raycast_result["normal"])

			z_world += step
		x_world += step

	var result := Models.TerrainScan.new()
	result.height_map = height_map
	result.normal_map = normal_map
	return result


func create_base_placement_map(scan_result: Models.TerrainScan) -> Image:
	var image_resolution = scan_result.height_map.image.get_size()
	var placement_map = Image.new()
	var placement_map_image_data = PackedByteArray()
	placement_map_image_data.resize(3 * image_resolution.x * image_resolution.y)
	placement_map.set_data(image_resolution.x, image_resolution.x, false, Image.FORMAT_RGB8, placement_map_image_data)

	for x in image_resolution.x:
		for z in image_resolution.y:
			var can_place_anything = true

			var normal := scan_result.normal_map._get_pixel(x, z)
			var height := scan_result.height_map._get_pixel(x, z)
			if normal.angle_to(Vector3.UP) > max_angle_radians:
				can_place_anything = false
			elif height.y == 0:
				can_place_anything = false
			elif height.y < -0.27:
				can_place_anything = false # todo - avoid placing grass on sand
			else:
				for neighbor in scan_result.height_map._dbg_get_neighbors(x, z, cliff_detection_skirt_samples):
					var height_diff = height.y - neighbor.y
					if height_diff > cliff_detection_threshold:
						can_place_anything = false
						break

			if can_place_anything:
				placement_map.set_pixel(x, z, Color.GREEN)

	return placement_map


func spawn_grass(placement_map: Image, scan_result: Models.TerrainScan):
	var image_resolution = scan_result.height_map.image.get_size()

	var d_rotation_min = -max_rotation_radians / 2
	var d_rotation_max = max_rotation_radians / 2
	var d_tilt_min = -max_tilt_radians / 2
	var d_tilt_max = max_tilt_radians / 2
	var d_scale_min = object_scale - max_scale_change
	var d_scale_max = object_scale + max_scale_change

	var instances = []
	for x in image_resolution.x:
		for y in image_resolution.y:
			var placement = placement_map.get_pixel(x, y)
			if placement != Color.GREEN:
				continue

			var instance_position := scan_result.height_map._get_pixel(x, y) - position
			var instance_transform = Transform3D(Basis(), instance_position)
			instance_transform = instance_transform.rotated_local(Vector3.FORWARD, randf_range(d_tilt_min, d_tilt_max))
			instance_transform = instance_transform.rotated_local(Vector3.RIGHT, randf_range(d_tilt_min, d_tilt_max))
			instance_transform = instance_transform.rotated_local(Vector3.UP, randf_range(d_rotation_min, d_rotation_max))
			instance_transform = instance_transform.scaled_local(object_scale * Vector3.ONE * randf_range(d_scale_min, d_scale_max))
			instances.append(instance_transform)

	multimesh.instance_count = 0
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.instance_count = len(instances)

	for i in range(len(instances)):
		multimesh.set_instance_transform(i, instances[i])

