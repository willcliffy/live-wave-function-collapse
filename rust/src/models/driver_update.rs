use godot::prelude::*;

use super::{manager::ManagerState, prototype::Prototype};

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
pub struct ManagerUpdate {
    pub new_state: Option<ManagerState>,
    pub changes: Option<Vec<CellChangeGodot>>,
}

impl ManagerUpdate {
    pub fn new(new_state: Option<ManagerState>, changes: Option<Vec<CellChangeGodot>>) -> Self {
        Self { new_state, changes }
    }

    pub fn new_state(new_state: ManagerState) -> Self {
        ManagerUpdate::new(Some(new_state), None)
    }

    pub fn new_changes(changes: Vec<CellChange>) -> Self {
        ManagerUpdate::new(
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
