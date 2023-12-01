extends Node

class Feature:
	var Resolution: Vector2
	var PlacementMap: ImageTexture
	var RotationMap: ImageTexture
	var ScaleMap: ImageTexture

class FeatureBuilder:
	var HeightMap: ImageTexture
	var Resolution: Vector2
	var Scale: float
	var ScaleRange: Vector2
	var NormalAngleRange: Vector2
	var PlacementNoise: ImageTexture
	var PlacementThreshold: float

	func build() -> Feature:
		var image := Image.new()
		var hmap_size := HeightMap.get_size()
		for x in hmap_size.x:
			for y in hmap_size.y:
				pass
		return null
