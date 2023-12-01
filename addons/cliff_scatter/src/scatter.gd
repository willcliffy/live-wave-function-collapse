@tool
extends MultiMeshInstance3D

signal rebuild()

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

@export_range(0.0, 100.0)
var tile_resolution: float = 8.0:
	set(val):
		tile_resolution = val
		if _is_ready: _build_queued = true

@export_range(0, 360)
var max_angle_degrees: float = 45:
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

@export var cliff_detection_skirt:  float = 0.2:
	set(val):
		cliff_detection_skirt = val
		if _is_ready: _build_queued = true

@export_group("Features")

@export
var features: Array = []

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
	var maps = _scan_terrain()
	var heightmap_image = maps[0]
	var normalmap_image = maps[1]
	ResourceSaver.save(heightmap_image, "res://_game/heightmap.tres")
	ResourceSaver.save(normalmap_image, "res://_game/normalmap.tres")


func _scan_terrain() -> Array:
	print("building height and normal maps")
	var space_state = get_world_3d().get_direct_space_state()

	# offset the xz coordinates slightly for raycasts to avoid getting stuck in gaps between modules
	# Maybe one day combine the modules into one mesh and recreate the collision shape, but until then this is fine
	var raycast_xz_offset = 0.001

	var map_width = (shape_size.x - 1) * tile_resolution
	var map_length = (shape_size.z - 1) * tile_resolution

	var heightmap_image = Image.new()
	var heightmap_image_data = PackedByteArray()
	heightmap_image_data.resize(3 * map_width * map_length)
	heightmap_image.set_data(map_width, map_length, false, Image.FORMAT_RGB8, heightmap_image_data)

	var normalmap_image = Image.new()
	var normalmap_image_data = PackedByteArray()
	normalmap_image_data.resize(3 * map_width * map_length)
	normalmap_image.set_data(map_width, map_length, false, Image.FORMAT_RGB8, normalmap_image_data)

	for x in range(0, shape_size.x - 1):
		for i in range(0, tile_resolution):
			var x_pixel = x * tile_resolution + i
			for z in range(0, shape_size.z - 1):
				for j in range(0, tile_resolution):
					var z_pixel = z * tile_resolution + j

					var cast_origin = Vector3(
						x + float(i) / tile_resolution + raycast_xz_offset,
						shape_size.y + 1.0,
						z + float(j) / tile_resolution + raycast_xz_offset)
					var cast_end = cast_origin + Vector3.DOWN * (shape_size.y + 2.0)
					var query = PhysicsRayQueryParameters3D.create(cast_origin, cast_end)
					query.collide_with_areas = true

					var raycast_result = space_state.intersect_ray(query)
					if raycast_result == null or not "collider" in raycast_result:
						continue

					var pos: Vector3 = raycast_result["position"]
					var height = (pos.y + 1) / shape_size.y
					var pos_color: Color = Color(height, height, height)
					heightmap_image.set_pixel(x_pixel, z_pixel, pos_color)

					var norm: Vector3 = raycast_result["normal"]
					var norm_color: Color = Color(norm.x, norm.y, norm.z)
					normalmap_image.set_pixel(x_pixel, z_pixel, norm_color)

	return [heightmap_image, normalmap_image]

	#var d_rotation_min = -max_rotation_radians / 2
	#var d_rotation_max = max_rotation_radians / 2
	#var d_tilt_min = -max_tilt_radians / 2
	#var d_tilt_max = max_tilt_radians / 2
	#var d_scale_min = 1 - max_scale_change
	#var d_scale_max = 1 + max_scale_change
	
	#instance_transform = instance_transform.rotated_local(Vector3.FORWARD, randf_range(d_tilt_min, d_tilt_max))
	#instance_transform = instance_transform.rotated_local(Vector3.RIGHT, randf_range(d_tilt_min, d_tilt_max))
	#instance_transform = instance_transform.rotated_local(Vector3.UP, randf_range(d_rotation_min, d_rotation_max))
	#instance_transform = instance_transform.scaled_local(object_scale * Vector3.ONE * randf_range(d_scale_min, d_scale_max))
	#working_map[w_x][w_z] = instance_transform
	#instance_count += 1
	#print("skipped ", skipped, " for a instance count of ", instance_count)

	#for x in range(len(working_map)):
		#var row = working_map[x]
		#for z in range(len(row)):
			#if working_map[x][z] == null: continue
			#var instance = working_map[x][z]
			## Check neighbors within x +/- ? and z +/- ?
#
			#for x_n in range(x - offset, x + offset + 1):
				#var b = false
				#for z_n in range(z - 1, z + offset + 1):
					#if x_n < 0 or z_n < 0 or x_n >= len(working_map) or z_n >= len(working_map[x_n]):
						#continue
					#var other_instance = working_map[x_n][z_n]
					#if other_instance == null:
						#continue
					#if abs(other_instance.origin.y - instance.origin.y) > 0.35:
						#if other_instance.origin.y > instance.origin.y:
							#working_map[x_n][z_n] = null
						#else:
							#working_map[x][z] = null
						#b = true
						#break
				#if b: break
#
	#multimesh.transform_format = MultiMesh.TRANSFORM_3D
	#multimesh.instance_count = instance_count
#
	#var instance = 0
	#for row in working_map:
		#for instance_transform in row:
			#if instance_transform == null: continue
			#result.append(instance_transform)
			#multimesh.set_instance_transform(instance, instance_transform)
			#instance += 1
	#print("scanned, placed ", instance, " instances. Instance count: ", instance_count)
