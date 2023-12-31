import bpy

TOLERANCE = 0.1
MODULE_RADIUS = 5


def main():
    objs = bpy.context.selected_objects

    module_radius = MODULE_RADIUS
    tolerance = TOLERANCE

    for obj in objs:
        correct_y_on_neg_x = []

        for vert in sorted(obj.data.vertices, key=lambda v: v.co.y):
            on_x_edge_1 = vert.co.x > -module_radius - tolerance \
                and vert.co.x < -module_radius + tolerance
            if on_x_edge_1:
                correct_y_on_neg_x.append(vert)

        y_i = -module_radius
        step = (2 * module_radius) / (len(correct_y_on_neg_x) - 1)
        for vert in correct_y_on_neg_x:
            new_location = vert.co
            new_location.y = y_i
            vert.co = new_location
            y_i += step


if __name__ == "__main__":
    main()
