extends Node


class TerrainScan:
	var height_map: WorldSpaceMapRGB
	var normal_map: WorldSpaceMapRGB


class WorldSpaceMapRGB:
	const _world_floor := 1.0

	var image: Image
	
	var region_position: Vector3
	var region_image_size: Vector2
	var region_world_size: Vector3

	var samples_per_meter: int

	func initialize(region_position: Vector3, region_size: Vector3, samples_per_meter: int, image_format: Image.Format = Image.FORMAT_RGBF):
		self.region_position = region_position
		self.region_world_size = region_size
		self.region_image_size = samples_per_meter * Vector2(region_world_size.x - 1, region_world_size.z - 1)
		self.samples_per_meter = samples_per_meter

		var image_data = PackedByteArray()
		var image_data_size = region_image_size.x * region_image_size.y
		if image_format == Image.FORMAT_RGB8:
			image_data_size *= 3
		elif image_format == Image.FORMAT_RGBF:
			image_data_size *= 12
		else:
			print("Unsupported image format!! ", image_format)
		image_data.resize(image_data_size)

		self.image = Image.new()
		self.image.set_data(region_image_size.x, region_image_size.y, false, image_format, image_data)

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
		var x_image = floor((x_world - region_position.x) * samples_per_meter)
		var y_image = floor((z_world - region_position.z) * samples_per_meter)
		return Vector2i(x_image, y_image)

	func _convert_image_space_to_world_space(x_image: int, y_image: int) -> Vector2:
		var x_world = (float(x_image) / float(samples_per_meter)) + region_position.x
		var z_world = (float(y_image) / float(samples_per_meter)) + region_position.z
		return Vector2(x_world, z_world)

	func _dbg_get_neighbors(x_image: int, z_image: int, neighborhood: int) -> Array:
		var neighbors = []
		for x_i in range(x_image - neighborhood, x_image + neighborhood + 1):
			if x_i < 0 or x_i > region_image_size.x:
				continue
			for z_i in range(z_image - neighborhood, z_image + neighborhood + 1):
				if z_i < 0 or z_i > region_image_size.y:
					continue
				neighbors.append(_get_pixel(x_i, z_i))

		return neighbors
