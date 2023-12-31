
import bpy
import bmesh
import json
import math
import os
from mathutils import Vector, Matrix

BOUNDARY_TOLERANCE = 0.001
MESH_NAME = "wfc_module_"

PROTO_NAME = "mesh_name"
PROTO_ROTATION = "mesh_rotation"
PROTO_NEIGHBOURS = "valid_neighbours"
PROTO_pX = "posX"
PROTO_nX = "negX"
PROTO_pY = "posY"
PROTO_nY = "negY"
PROTO_pZ = "posZ"
PROTO_nZ = "negZ"
PROTO_FACES = [PROTO_pX, PROTO_pY, PROTO_nX, PROTO_nY, PROTO_pZ, PROTO_nZ]
PROTO_CONSTRAIN_TO = "constrain_to"
PROTO_CONSTRAIN_FROM = "constrain_from"
PROTO_WEIGHT = "weight"
PROTO_NO_ID = "no_id"
PROTO_NO_ID_SYM = "no_id_sym"
PROTO_CUSTOM_ATTRIBUTES = {
    PROTO_CONSTRAIN_TO: "",
    PROTO_CONSTRAIN_FROM: "",
    PROTO_WEIGHT: 10,
    PROTO_NO_ID: 0,
    PROTO_NO_ID_SYM: 0,
}

JSON_FILE = "prototype_data.json"
MODULES_FILE = "wfc_modules"

pX = 0
pY = 1
nX = 2
nY = 3
pZ = 4
nZ = 5


def export_selection():
    objs = bpy.context.selected_objects
    
    module_meshes = hash_boundaries(objs)  # Duplicate modules, find boundaries, assign names for each
    prototypes = create_module_prototypes(module_meshes)  # Create profiles for all modules plus rotated variations
    add_adjacency_lists(prototypes)  # Add lists of valid neighbours to prototypes
    write_protoype_json(prototypes)
    export_modules(module_meshes)
    bpy.ops.object.delete(use_global=False)  # Comment this out to keep the exported meshes in the scene


def hash_boundaries(objs):
    bi = 0  # Used to name boundaries, incremented whenever a new one is found
    vbi = 0  # Used to name vertical boundaries, incremented whenever a new one is found
    modules = list()
    boundary_dict = dict()
    vertical_boundary_dict = dict()

    for obj in objs:
        copy = duplicate_mesh(obj)  # remove copy? do we need duplicate data?
        move_to_origin(copy)
        modules.append(copy)
        copy_boundaries = get_horizontal_boundaries(copy)
        vertical_boundaries = get_vertical_boundaries(copy)

        for i, boundary in enumerate(copy_boundaries):
            if len(boundary) == 0:
                copy.data[str(i)] = "-1"
                continue

            existing_boundary = boundary_exists(boundary, boundary_dict)

            if existing_boundary:
                copy.data[str(i)] = existing_boundary
            else:
                # Create new boundary entry and name it
                new_name = str(bi)
                if compare_boundaries(boundary, flip_boundary(boundary)):  # if symmetrical
                    new_name += "s"
                    boundary_dict[new_name] = boundary
                else:
                    boundary_dict[new_name] = boundary
                    boundary_dict[new_name + "f"] = flip_boundary(boundary)
                copy.data[str(i)] = new_name
                bi += 1

        for i, boundary in enumerate(vertical_boundaries):
            if len(boundary) == 0:
                copy.data[str(4 + i)] = "-1"
                continue

            existing_boundary = boundary_exists(boundary, vertical_boundary_dict)

            if existing_boundary:
                copy.data[str(4 + i)] = existing_boundary
            else:
                # Create new boundary entry plus all rotations
                new_name = "v" + str(vbi)
                vertical_boundary_dict[new_name + "_0"] = boundary
                rotated_boundaries = get_rotated_boundaries(boundary)
                vertical_boundary_dict[new_name + "_1"] = rotated_boundaries[2]
                vertical_boundary_dict[new_name + "_2"] = rotated_boundaries[1]
                vertical_boundary_dict[new_name + "_3"] = rotated_boundaries[0]
                copy.data[str(4 + i)] = new_name + "_0"
                vbi += 1

        bpy.context.collection.objects.link(copy)  # link to scene in case we want to see what we exported

    return modules


def create_module_prototypes(modules):
    all_prototypes = dict()
    mi = 0
    pi = 0

    for module in modules:
        prototypes = list()

        for i in range(4):
            prototype = dict()
            prototype[PROTO_NAME] = MESH_NAME + str(mi)
            prototype[PROTO_ROTATION] = i
            prototype[PROTO_pX] = module.data[str((pX + (i * 3)) % 4)]
            prototype[PROTO_nX] = module.data[str((nX + (i * 3)) % 4)]
            prototype[PROTO_pY] = module.data[str((pY + (i * 3)) % 4)]
            prototype[PROTO_nY] = module.data[str((nY + (i * 3)) % 4)]
            prototype[PROTO_pZ] = rotate_vertical_boundary(module.data[str(pZ)], i)
            prototype[PROTO_nZ] = rotate_vertical_boundary(module.data[str(nZ)], i)

            prototype = _add_custom_constraints(prototype, module)

            all_prototypes["{}{}".format("p", pi)] = prototype
            pi += 1
            if "NO_SYM" in module.name:
                break
        module.name = MESH_NAME + str(mi) + "_col"
        mi += 1

    all_prototypes["p-1"] = _blank_prototype()

    return all_prototypes


def _add_custom_constraints(prototype, module):
    for attribute, default in PROTO_CUSTOM_ATTRIBUTES.items():
        try:
            custom_constraint = module.data[attribute]
            prototype[attribute] = custom_constraint
        except Exception:
            prototype[attribute] = default
    return prototype


def add_adjacency_lists(prototypes):
    for p_name, prototype in prototypes.items():
        valid_neighbours = list()
        no_id = PROTO_NO_ID in prototype and prototype[PROTO_NO_ID]
        no_id_sym = PROTO_NO_ID_SYM in prototype and prototype[PROTO_NO_ID_SYM]

        for i, face in enumerate(PROTO_FACES):
            boundary = prototype[face]
            valid_boundary_neighbours = list()

            for other_p_name, other_prototype in prototypes.items():
                if prototype[PROTO_NAME] == other_prototype[PROTO_NAME]:
                    continue
                    # if no_id:
                    #     continue
                    # rotation_distance = abs(prototype[PROTO_ROTATION] - other_prototype[PROTO_ROTATION])
                    # if no_id_sym and rotation_distance == 2:
                    #     continue

                if i < 4:
                    other_boundary = other_prototype[PROTO_FACES[(i + 2) % 4]]
                elif i == 4:
                    other_boundary = other_prototype[PROTO_FACES[5]]
                else:
                    other_boundary = other_prototype[PROTO_FACES[4]]

                valid = boundaries_are_valid(boundary, other_boundary)
                if valid:
                    valid_boundary_neighbours.append(other_p_name)

            valid_neighbours.append(valid_boundary_neighbours)
        prototype[PROTO_NEIGHBOURS] = valid_neighbours


def write_protoype_json(prototypes):
    blend_file_path = bpy.data.filepath
    directory = os.path.dirname(blend_file_path)
    json_path = "{}/{}".format(directory, JSON_FILE)

    with open(json_path, "w") as outfile:
        json.dump(prototypes, outfile, indent="  ")


def export_modules(objs):
    for obj in bpy.context.selected_objects:
        obj.select_set(False)
    for obj in objs:
        obj.select_set(True)

    blend_file_path = bpy.data.filepath
    directory = os.path.dirname(blend_file_path)
    filepath = "{}/{}".format(directory, MODULES_FILE)

    bpy.ops.export_scene.gltf(filepath=filepath, use_selection=True, export_normals=True)


def boundaries_are_valid(boundary, other_boundary):
    if boundary == "-1f" and other_boundary == "-1f":
        return True
    elif boundary.endswith("f"):
        if boundary.rpartition("f")[0] == other_boundary:
            return True
    elif other_boundary.endswith("f"):
        if other_boundary.rpartition("f")[0] == boundary:
            return True
    elif boundary == other_boundary:
        if boundary.endswith("s") or boundary.startswith("v"):
            return True
    else:
        return False


def rotate_vertical_boundary(name, rotation):
    if name == "-1":
        return "-1"
    rot = int(name.rpartition("_")[2])
    new_rot = (rot + rotation) % 4
    new_name = "{N}_{R}".format(N=name.rpartition("_")[0], R=new_rot)
    return new_name


def get_horizontal_boundaries(obj):
    boundaries = [list(), list(), list(), list()]  # +X, +Y, -X, -Y
    bm = bmesh.new()
    bm.from_mesh(obj.data)

    for i in range(4):
        for vert in bm.verts:
            if vert.co.x >= 0.5 - BOUNDARY_TOLERANCE:
                nice_pos = round_position(vert.co, 4)
                boundaries[i].append(nice_pos)
        rotate_mesh_90(bm)

    return boundaries


def get_vertical_boundaries(obj):
    boundaries = [list(), list()]  # +X, +Y, -X, -Y
    bm = bmesh.new()
    bm.from_mesh(obj.data)

    rotate_mesh(bm, axis=Vector([0.0, 1.0, 0.0]), degrees=90.0)

    for i in range(2):
        for vert in bm.verts:
            if vert.co.x >= 0.5 - BOUNDARY_TOLERANCE:
                nice_pos = round_position(vert.co, 4)
                if i > 0:
                    nice_pos.y = -nice_pos.y
                boundaries[i].append(nice_pos)
        rotate_mesh(bm, axis=Vector([0.0, 1.0, 0.0]), degrees=180.0)
        rotate_mesh(bm, axis=Vector([1.0, 0.0, 0.0]), degrees=180.0)

    return boundaries


def boundary_exists(boundary, boundary_dict):
    if len(boundary_dict.keys()) == 0:
        return False
    for boundary_name, existing_boundary in boundary_dict.items():
        if compare_boundaries(boundary, existing_boundary):
            return boundary_name
    return False


def compare_boundaries(b1, b2, tolerance=0.1):
    # Function to check if a vector is within tolerance of any vector in a list
    def close_to_any(vec, vec_list, tolerance):
        return any(math.dist(v, vec) <= tolerance for v in vec_list)

    # Compare two boundaries and return True if they're similar within a tolerance
    # Compare two boundaries and return True if they're similar within a tolerance
    if len(b1) == len(b2):
        return all(close_to_any(v, b2, tolerance) for v in b1)
    elif len(b1) > len(b2):
        return all(close_to_any(v, b1, tolerance) for v in b2)
    else:
        return all(close_to_any(v, b2, tolerance) for v in b1)


def flip_boundary(boundary):
    new_boundary = list()
    for vert in boundary:
        new_vert = vert.copy()
        new_vert.y *= -1.0
        new_boundary.append(new_vert)
    return new_boundary


def get_boundary_contribution_from_vert(vert):
    boundary_list = [None, None, None, None, None, None]
    if vert.co.x >= 0.5 - BOUNDARY_TOLERANCE:
        boundary_list[0] = vert.co
    if vert.co.x <= -0.5 + BOUNDARY_TOLERANCE:
        boundary_list[1] = vert.co
    if vert.co.y >= 0.5 - BOUNDARY_TOLERANCE:
        boundary_list[2] = vert.co
    if vert.co.y <= -0.5 + BOUNDARY_TOLERANCE:
        boundary_list[3] = vert.co
    if vert.co.z >= 0.5 - BOUNDARY_TOLERANCE:
        boundary_list[4] = vert.co
    if vert.co.z <= -0.5 + BOUNDARY_TOLERANCE:
        boundary_list[5] = vert.co
    return boundary_list


def round_position(vec, i):
    result = Vector([0.0, 0.0, 0.0])
    result.x = round(vec.x, i)
    result.y = round(vec.y, i)
    result.z = round(vec.z, i)
    return result


def get_rotated_boundaries(boundary):
    boundaries = list()
    temp = list()
    for i in range(3):
        if not temp:
            temp = rotate_boundary(boundary)
        else:
            temp = rotate_boundary(temp)
        boundaries.append(temp)
    return boundaries


def rotate_boundary(boundary):
    rotated_boundary = list()
    for v in boundary:
        new_v = Vector([0.0, 0.0, 0.0])
        new_v.x = v.x
        new_v.y = v.z
        new_v.z = -v.y
        rotated_boundary.append(new_v)
    return rotated_boundary


def move_to_origin(obj):
    loc = obj.location
    obj.location = Vector([0.0, 0.0, 0.0])
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.translate(bm, vec=[-loc.x, -loc.y, -loc.z], verts=bm.verts)
    bm.to_mesh(obj.data)


def rotate_mesh_90(bm):
    rot = Matrix.Rotation(math.radians(-90), 4, Vector([0.0, 0.0, 1.0]))
    bmesh.ops.rotate(bm, cent=Vector([0.0, 0.0, 0.0]), matrix=rot, verts=bm.verts)


def rotate_mesh(bm, axis, degrees):
    rot = Matrix.Rotation(math.radians(degrees), 4, axis)
    bmesh.ops.rotate(bm, cent=Vector([0.0, 0.0, 0.0]), matrix=rot, verts=bm.verts)


def duplicate_mesh(obj):
    dup = obj.copy()
    dup.data = obj.data.copy()
    return dup


def _blank_prototype():
    proto = dict()
    proto[PROTO_NAME] = "-1"
    proto[PROTO_ROTATION] = 0
    proto[PROTO_pX] = "-1f"
    proto[PROTO_nX] = "-1f"
    proto[PROTO_pY] = "-1f"
    proto[PROTO_nY] = "-1f"
    proto[PROTO_pZ] = "-1f"
    proto[PROTO_nZ] = "-1f"
    proto[PROTO_CONSTRAIN_TO] = "-1"
    proto[PROTO_CONSTRAIN_FROM] = "-1"
    proto[PROTO_WEIGHT] = 1
    return proto


if __name__ == "__main__":
    export_selection()
