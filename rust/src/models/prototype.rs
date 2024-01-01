use std::{fs::File, io::Read};

use godot::{builtin::Vector3i, log::godot_print};
use serde::Deserialize;
use serde_json::{json, Value};

const P_X: usize = 0;
const P_Y: usize = 1;
const N_X: usize = 2;
const N_Y: usize = 3;
const P_Z: usize = 4;
const N_Z: usize = 5;

#[derive(PartialEq, Clone, Debug, Deserialize)]
pub struct Prototype {
    pub id: String,
    pub mesh_name: String,
    pub mesh_rotation: i32,
    pub pos_x: String,
    pub neg_x: String,
    pub pos_y: String,
    pub neg_y: String,
    pub pos_z: String,
    pub neg_z: String,
    pub constrain_to: String,
    pub constrain_from: String,
    pub weight: f32,
    pub no_id: i32,
    pub no_id_sym: i32,
    pub valid_neighbors: Vec<Vec<String>>,
}

impl Prototype {
    fn from_json_value(id: String, json: &Value) -> Option<Self> {
        match json {
            Value::Object(obj) => {
                let mesh_name = obj.get("mesh_name")?.as_str()?.to_string();
                let mesh_rotation = obj.get("mesh_rotation")?.as_i64()? as i32;
                let pos_x = obj.get("posX")?.as_str()?.to_string();
                let neg_x = obj.get("negX")?.as_str()?.to_string();
                let pos_y = obj.get("posY")?.as_str()?.to_string();
                let neg_y = obj.get("negY")?.as_str()?.to_string();
                let pos_z = obj.get("posZ")?.as_str()?.to_string();
                let neg_z = obj.get("negZ")?.as_str()?.to_string();
                let constrain_to = obj.get("constrain_to")?.as_str()?.to_string();
                let constrain_from = obj.get("constrain_from")?.as_str()?.to_string();
                let weight = obj.get("weight")?.as_f64()? as f32;
                let no_id = obj.get("no_id").or(Some(&json!(0)))?.as_i64()? as i32;
                let no_id_sym = obj.get("no_id_sym").or(Some(&json!(0)))?.as_i64()? as i32;

                let neighbors_json = obj.get("valid_neighbours")?.as_array()?;

                let mut valid_neighbors = vec![];
                for neighbor_value in neighbors_json {
                    if let Some(neighbors) = neighbor_value.as_array() {
                        let mut valid_neighbor_row = vec![];
                        for neighbor in neighbors {
                            match neighbor.as_str() {
                                Some(neighbor_string) => {
                                    valid_neighbor_row.push(neighbor_string.to_string())
                                }
                                None => godot_print!("skipping neighbor as it's not a string!"),
                            }
                        }
                        valid_neighbors.push(valid_neighbor_row);
                    }
                }
                // godot_print!("{:?}", valid_neighbors);

                Some(Prototype {
                    id,
                    mesh_name,
                    mesh_rotation,
                    pos_x,
                    neg_x,
                    pos_y,
                    neg_y,
                    pos_z,
                    neg_z,
                    constrain_to,
                    constrain_from,
                    weight,
                    no_id,
                    no_id_sym,
                    valid_neighbors,
                })
            }
            _ => None,
        }
    }

    pub fn load() -> Vec<Prototype> {
        let mut file = File::open("prototype_data.json").expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");

        let mut protos: Vec<Prototype> = vec![];
        let parsed: Value = serde_json::from_str(&contents).unwrap_or_default();

        if let Some(obj) = parsed.as_object() {
            for (key, value) in obj.iter() {
                // Attempt to parse each value into a MyStruct
                if let Some(parsed_struct) = Prototype::from_json_value(key.to_string(), value) {
                    protos.push(parsed_struct)
                } else {
                    godot_print!("failed to parse Prototype '{}', ignoring", key);
                }
            }
        } else {
            println!("The parsed JSON is not an object");
        }

        protos
    }

    pub fn compatible_with(&self, other_id: String, direction: Vector3i) -> bool {
        let direction_index = match direction {
            Vector3i::UP => P_Z,
            Vector3i::DOWN => N_Z,
            Vector3i::RIGHT => P_X,
            Vector3i::LEFT => N_X,
            Vector3i::FORWARD => P_Y,
            Vector3i::BACK => N_Y,
            Vector3i { x: _, y: _, z: _ } => return false,
        };

        if let Some(neighbors) = self.valid_neighbors.get(direction_index) {
            if neighbors.contains(&other_id) {
                return true;
            }
        }

        false
    }

    pub fn compatible_with_any(&self, others: &Vec<Prototype>, direction: Vector3i) -> bool {
        others
            .iter()
            .any(|p| self.compatible_with(p.id.clone(), direction))
    }
}
