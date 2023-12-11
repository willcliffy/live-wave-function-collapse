use godot::prelude::*;

use super::collapser_state::CollapserState;

#[derive(ToGodot, FromGodot, GodotConvert, Debug)]
pub struct SlotChange {
    pub position: Vector3,
    pub new_protos: String,
}

#[derive(GodotClass, Debug)]
pub struct DriverUpdate {
    pub new_state: Option<CollapserState>,
    pub changes: Option<Vec<SlotChange>>,
}

impl DriverUpdate {
    pub fn new(new_state: Option<CollapserState>, changes: Option<Vec<SlotChange>>) -> Self {
        Self { new_state, changes }
    }

    pub fn new_state(new_state: CollapserState) -> Self {
        DriverUpdate::new(Some(new_state), None)
    }

    pub fn new_changes(changes: Vec<SlotChange>) -> Self {
        DriverUpdate::new(None, Some(changes))
    }
}
