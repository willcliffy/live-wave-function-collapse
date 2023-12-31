import bpy

TOLERANCE = 0.001
MODULE_RADIUS = 5


def main():
    objs = bpy.context.selected_objects

    module_radius = MODULE_RADIUS
    tolerance = TOLERANCE

    for obj in objs:
        correct_x_on_neg_y = []
        correct_x_on_pos_y = []
        correct_y_on_neg_x = []
        correct_y_on_pos_x = []

        for vert in obj.data.vertices:
            on_neg_x_edge = vert.co.x > -module_radius - \
                tolerance and vert.co.x < -module_radius + tolerance
            on_pos_x_edge = vert.co.x > module_radius - \
                tolerance and vert.co.x < module_radius + tolerance
            on_neg_y_edge = vert.co.y > -module_radius - \
                tolerance and vert.co.y < -module_radius - tolerance
            on_pos_y_edge = vert.co.y > module_radius - \
                tolerance and vert.co.y < module_radius + tolerance
            if not (on_neg_x_edge or on_pos_x_edge or on_neg_y_edge or on_pos_y_edge):
                continue

            new_location = vert.co
            if on_neg_x_edge or on_pos_x_edge:
                new_location.x = module_radius * (-1 if vert.co.x < 0 else 1)
            else:
                if on_neg_y_edge:
                    correct_x_on_neg_y.append(vert)
                elif on_pos_y_edge:
                    correct_x_on_pos_y.append(vert)

            if on_neg_y_edge or on_pos_y_edge:
                new_location.y = module_radius * (-1 if vert.co.y < 0 else 1)
            else:
                if on_neg_x_edge:
                    correct_y_on_neg_x.append(vert)
                elif on_pos_x_edge:
                    correct_y_on_pos_x.append(vert)

            new_location.z = 0
            vert.co = new_location

        sorted(correct_y_on_neg_x, key=lambda vert: vert.co.y, reverse=True)
        y_step = 2 * module_radius / (len(correct_y_on_neg_x) + 2 + 1)
        y_i = -module_radius
        for vert in correct_y_on_neg_x:
            y_i += y_step
            vert.co.y = y_i

        # sorted(correct_y_on_pos_x, key=lambda vert: vert.co.y)
        # y_step = 2 * module_radius / (len(correct_y_on_pos_x) + 2 + 1)
        # y_i = module_radius
        # for vert in correct_y_on_pos_x:
        #     y_i -= y_step
        #     vert.co.y = y_i


if __name__ == "__main__":
    main()
