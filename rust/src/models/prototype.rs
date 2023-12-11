#[derive(PartialEq, Clone, Debug)]
pub struct Prototype {
    pub id: String,
    pub mesh_name: String,
    pub mesh_rotation: i8,
    pub pos_x: String,
    pub neg_x: String,
    pub pos_y: String,
    pub neg_y: String,
    pub pos_z: String,
    pub neg_z: String,
    pub constrain_to: String,
    pub constrain_from: String,
    pub weight: i32,
    pub no_id: i32,
    pub no_id_sym: i32,
    pub valid_neighbors: Vec<Vec<String>>,
}
