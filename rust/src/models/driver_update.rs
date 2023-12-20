use godot::prelude::*;

use super::{collapser_state::CollapserState, prototype::Prototype};

#[derive(Debug, Clone)]
pub struct CellChange {
    pub position: Vector3i,
    pub new_protos: Vec<Prototype>,
}

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

#[derive(GodotClass, Debug)]
pub struct DriverUpdate {
    pub new_state: Option<CollapserState>,
    pub changes: Option<Vec<CellChangeGodot>>,
}

impl DriverUpdate {
    pub fn new(new_state: Option<CollapserState>, changes: Option<Vec<CellChangeGodot>>) -> Self {
        Self { new_state, changes }
    }

    pub fn new_state(new_state: CollapserState) -> Self {
        DriverUpdate::new(Some(new_state), None)
    }

    pub fn new_changes(changes: Vec<CellChange>) -> Self {
        DriverUpdate::new(
            None,
            Some(
                changes
                    .iter()
                    .map(|c| CellChangeGodot::from_internal(c.position, c.new_protos.clone()))
                    .collect(),
            ),
        )
    }
}
