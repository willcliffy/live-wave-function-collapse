import bpy


MODULE_SIZE = 10
MOVE_DISTANCE_X = -5
MOVE_DISTANCE_Y = 0
MOVE_DISTANCE_Z = -5


def main():
    objs = bpy.context.selected_objects

    for obj in objs:
        world_matrix = obj.matrix_world
        for vert in obj.data.vertices:
            new_location = vert.co
            new_location.x += MOVE_DISTANCE_X
            new_location.y += MOVE_DISTANCE_Y
            new_location.z += MOVE_DISTANCE_Z
            vert.co = new_location


if __name__ == "__main__":
    main()
