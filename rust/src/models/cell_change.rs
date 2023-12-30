use godot::prelude::*;

use super::prototype::Prototype;

#[derive(ToGodot, FromGodot, GodotConvert, Debug)]
pub struct CellChangeGodot {
    pub position: Vector3i,
    pub new_protos: String,
}

impl CellChangeGodot {
    pub fn from_internal(position: Vector3i, protos: Vec<Prototype>) -> Self {
        let mut new_protos: String = protos.iter().map(|p| p.id.clone() + ",".into()).collect();
        new_protos = new_protos.trim_end_matches(",").into();
        Self {
            position,
            new_protos,
        }
    }
}
