use godot::prelude::*;

use super::{collapser_state::CollapserState, prototype::Prototype};

#[derive(Debug)]
pub struct SlotChange {
    pub position: Vector3,
    pub new_protos: Vec<Prototype>,
}

#[derive(ToGodot, FromGodot, GodotConvert, Debug)]
pub struct SlotChangeGodot {
    pub position: Vector3,
    pub new_protos: String,
}

// #[godot_api]
impl SlotChangeGodot {
    pub fn from_internal(position: Vector3, protos: Vec<Prototype>) -> Self {
        let mut new_protos: String = protos.iter().map(|p| p.id.clone() + ",".into()).collect();
        new_protos = new_protos.trim_end_matches(",").into();
        Self {
            position,
            new_protos,
        }
    }

    // #[func]
    // fn unpack_protos(&mut self) -> Array<GString> {
    //     let mut changes = Array::new();
    //     self.new_protos
    //         .split(",")
    //         .for_each(|p| changes.push(p.into_godot()));
    //     changes
    // }
}

#[derive(GodotClass, Debug)]
pub struct DriverUpdate {
    pub new_state: Option<CollapserState>,
    pub changes: Option<Vec<SlotChangeGodot>>,
}

impl DriverUpdate {
    pub fn new(new_state: Option<CollapserState>, changes: Option<Vec<SlotChangeGodot>>) -> Self {
        Self { new_state, changes }
    }

    pub fn new_state(new_state: CollapserState) -> Self {
        DriverUpdate::new(Some(new_state), None)
    }

    pub fn new_changes(changes: Vec<SlotChange>) -> Self {
        DriverUpdate::new(
            None,
            Some(
                changes
                    .iter()
                    .map(|c| SlotChangeGodot::from_internal(c.position, c.new_protos.clone()))
                    .collect(),
            ),
        )
    }
}
