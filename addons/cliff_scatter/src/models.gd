extends Node


class_name ScatterModels

class ScatterFeatureInput:
	var mesh: Mesh
	var sample_resolution: int # in samples/meter
	var placement_noise: FastNoiseLite

	var placement_noise_threshold: float = 0.25

	var avoid_cliffs: bool = true
	var max_normal_radians: float = PI / 2.0

	var scale: float = 1.0
	var scale_y: bool = true
	var scale_delta: float = 0.2

	var tilt_radians: float = 0.0
	var tilt_radians_delta: float = PI / 8.0

	var rotation_radians: float = 0
	var rotation_radians_delta: float = 2 * PI

	func sample_scale() -> Vector3:
		var d_scale_min := scale - scale_delta
		var d_scale_max := scale + scale_delta
		return Vector3.ONE * scale * (scale + randf_range(-scale_delta, scale_delta))

	func sample_rotation() -> Vector3:
		var d_rotation_min := -rotation_radians_delta / 2
		var d_rotation_max := rotation_radians_delta / 2
		var d_tilt_min := -tilt_radians_delta / 2
		var d_tilt_max := tilt_radians_delta / 2
		var result := Vector3(
			tilt_radians + randf_range(d_tilt_min, d_tilt_max),
			rotation_radians + randf_range(d_rotation_min, d_rotation_max),
			tilt_radians + randf_range(d_tilt_min, d_tilt_max),
		)

		return result

	func can_place(x: int, z: int):
		return placement_noise.get_noise_2d(x, z) > placement_noise_threshold


class ScatterFeature:
	var terrain: TerrainScan

	var placement_map: WorldSpaceMapRGB
	var rotation_map: WorldSpaceMapRGB
	var scale_map: WorldSpaceMapRGB

	var align_on_normals: bool = false
	var align_on_normals_ratio: float = 0.45

	func initialize(terrain_scan: TerrainScan, input: ScatterFeatureInput):
		terrain = terrain_scan

		placement_map = WorldSpaceMapRGB.new()
		placement_map.initialize(terrain)
		rotation_map = WorldSpaceMapRGB.new()
		rotation_map.initialize(terrain)
		scale_map = WorldSpaceMapRGB.new()
		scale_map.initialize(terrain)

		const TEMP_CLIFF_DETECTION_SKIRT_SAMPLES = 4.0
		const TEMP_CLIFF_DETECTION_THRESHOLD = 0.35
		const TEMP_CLIFF_DETECTION_SKIRT_METERS = 0.35

		var image_resolution := terrain.height_map.image.get_size()
		for x in image_resolution.x:
			for z in image_resolution.y:
				if not input.can_place(x, z):
					continue

				var instance_position := terrain.height_map._get_pixel(x, z)
				if instance_position.is_equal_approx(Vector3.ZERO):
					continue

				var instance_normal := terrain.normal_map._get_pixel(x, z)

				var instance_scale_override := 1.0

				if instance_normal.angle_to(Vector3.UP) > input.max_normal_radians:
					continue
				elif instance_position.y < -0.3:
					continue # todo - avoid placing grass on sand
				else:
					var can_place_anything := true
					for neighbor: Vector3 in terrain.height_map._dbg_get_neighbors(x, z, 2 * TEMP_CLIFF_DETECTION_SKIRT_SAMPLES):
						var height_diff := instance_position.y - neighbor.y
						if height_diff < TEMP_CLIFF_DETECTION_THRESHOLD:
							continue

						var pos_2d := Vector2(instance_position.x, instance_position.z)
						var neighbor_pos_2d := Vector2(neighbor.x, neighbor.z)
						var distance_2d := pos_2d.distance_to(neighbor_pos_2d)
						if distance_2d <= TEMP_CLIFF_DETECTION_SKIRT_METERS:
							can_place_anything = false
							break
						elif distance_2d <= 2.0 * TEMP_CLIFF_DETECTION_SKIRT_METERS:
							instance_scale_override = distance_2d / (2.0 * TEMP_CLIFF_DETECTION_SKIRT_METERS)
					if not can_place_anything:
						continue

				var rotation := input.sample_rotation()
				if align_on_normals:
					rotation += align_on_normals_ratio * instance_normal

				placement_map._set_pixel(x, z, Vector3.ONE)
				rotation_map._set_pixel(x, z, rotation)
				scale_map._set_pixel(x, z, input.sample_scale() * instance_scale_override)


class TerrainScan:
	var position: Vector3
	var shape: Vector3
	var resolution: float # in samples per meter

	var height_map: WorldSpaceMapRGB
	var normal_map: WorldSpaceMapRGB

	func initialize(in_position: Vector3, in_shape: Vector3, in_resolution: float):
		position = in_position
		shape = in_shape
		resolution = in_resolution


class WorldSpaceMapRGB:
	const _world_floor := 1.0

	var image: Image

	var region_position: Vector3
	var region_image_shape: Vector2
	var region_world_shape: Vector3

	var region_resolution: int # in samples per meter

	func initialize(parent: TerrainScan, image_format: Image.Format = Image.FORMAT_RGBF):
		region_position = parent.position
		region_world_shape = parent.shape
		region_image_shape = parent.resolution * Vector2(region_world_shape.x, region_world_shape.z)
		region_resolution =  parent.resolution

		var image_data = PackedByteArray()
		var image_data_size = region_image_shape.x * region_image_shape.y
		if image_format in [Image.FORMAT_R8, Image.FORMAT_L8]:
			pass # no change!
		if image_format in [Image.FORMAT_RGB8]:
			image_data_size *= 3
		if image_format in [Image.FORMAT_RF, Image.FORMAT_RGBA8]:
			image_data_size *= 4
		elif image_format in [Image.FORMAT_RGBF]:
			image_data_size *= 12
		else:
			print("Unsupported image format!! ", image_format)
		image_data.resize(image_data_size)

		image = Image.new()
		image.set_data(region_image_shape.x, region_image_shape.y, false, image_format, image_data)

	func _get_pixel(x_image: int, y_image: int) -> Vector3:
		var pixel := image.get_pixel(x_image, y_image)
		return Vector3(pixel.r, pixel.g, pixel.b)

	func _set_pixel(x_image: int, y_image: int, value: Vector3):
		var pixel := Color(value.x, value.y, value.z)
		image.set_pixel(x_image, y_image, pixel)

	func get_value(x_world: float, z_world: float) -> Vector3:
		var image_coords = _convert_world_space_to_image_space(x_world, z_world)
		return _get_pixel(image_coords.x, image_coords.y)

	func set_value(x_world: float, z_world: float, value: Vector3):
		var image_coords = _convert_world_space_to_image_space(x_world, z_world)
		_set_pixel(image_coords.x, image_coords.y, value)

	func _convert_world_space_to_image_space(x_world: float, z_world: float) -> Vector2i:
		var x_image = floor((x_world - region_position.x) * region_resolution)
		var y_image = floor((z_world - region_position.z) * region_resolution)
		return Vector2i(x_image, y_image)

	func _convert_image_space_to_world_space(x_image: int, y_image: int) -> Vector2:
		var x_world = (float(x_image) / float(region_resolution)) + region_position.x
		var z_world = (float(y_image) / float(region_resolution)) + region_position.z
		return Vector2(x_world, z_world)

	func _dbg_get_neighbors(x_image: int, z_image: int, neighborhood: int) -> Array:
		var neighbors = []
		for x_i in range(x_image - neighborhood, x_image + neighborhood + 1):
			if x_i < 0 or x_i >= region_image_shape.x:
				continue
			for z_i in range(z_image - neighborhood, z_image + neighborhood + 1):
				if z_i < 0 or z_i >= region_image_shape.y:
					continue
				neighbors.append(_get_pixel(x_i, z_i))

		return neighbors
