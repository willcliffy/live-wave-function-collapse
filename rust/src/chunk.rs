use godot::prelude::*;

use crate::{models::driver_update::SlotChange, slot::Slot};

pub struct Chunk {
    _position: Vector3,

    _all_slots_matrix: Vec<Vec<Vec<Slot>>>,
    _all_slots_vec: Vec<Slot>,

    _chunk_slots_matrix: Vec<Vec<Vec<Slot>>>,
    _chunk_slots_vec: Vec<Slot>,
}

impl Chunk {
    fn _new(
        _position: Vector3,
        _all_slots_matrix: Vec<Vec<Vec<Slot>>>,
        _all_slots_vec: Vec<Slot>,
    ) -> Self {
        // TODO - fill these
        let _chunk_slots_matrix = vec![];
        let _chunk_slots_vec = vec![];
        Self {
            _position,
            _all_slots_matrix,
            _all_slots_vec,
            _chunk_slots_matrix,
            _chunk_slots_vec,
        }
    }

    pub fn get_overlapping(&self, _other: &Chunk) -> Vec<Vector3> {
        vec![]
    }

    pub fn reset_slots(&mut self, _slots: Vec<Vector3>) {}

    pub fn get_neighboring(&self, _other: &Chunk) -> Vec<Vector3> {
        vec![]
    }

    pub fn propagate_from(&mut self, _slots: Vec<Vector3>) {}

    pub fn collapse_next(&mut self) -> Option<Vec<SlotChange>> {
        if let Some(mut slot) = self._select_lowest_entropy() {
            let change = slot.collapse(None);
            return match change {
                Some(change) => Some(self._propagate(change)),
                None => None,
            };
        }

        None
    }

    pub fn apply_custom_constraints(&mut self) {}

    fn _select_lowest_entropy(&mut self) -> Option<Slot> {
        None
    }

    fn _get_neighbors(self) -> Vec<Slot> {
        vec![]
    }

    fn _propagate(&mut self, slot: SlotChange) -> Vec<SlotChange> {
        self._propagate_recursive(slot, 0)
    }

    fn _propagate_recursive(&mut self, _slot: SlotChange, _depth: i64) -> Vec<SlotChange> {
        vec![]
    }
}
