use godot::{builtin::Vector3i, engine::utilities::ceili, log::godot_print};

use crate::{
    chunk::Chunk,
    models::{
        collapser_state::CollapserState,
        driver_update::{DriverUpdate, SlotChange},
        prototype::Prototype,
    },
    slot::Slot,
};

pub struct Map {
    slots: Vec<Vec<Vec<Slot>>>,
    chunks: Vec<Chunk>,
    chunk_overlap: i32,
    current_chunk: usize,
}

impl Map {
    pub fn new(
        size: Vector3i,
        chunk_size: Vector3i,
        chunk_overlap: i32,
        all_protos: Vec<Prototype>,
    ) -> Self {
        let slots = generate_slots(size, all_protos);

        let chunks = generate_chunks(size, chunk_size, chunk_overlap);
        if let Some(chunk) = chunks.get(0) {
            chunk.apply_custom_constraints();
        }

        Self {
            slots,
            chunks,
            chunk_overlap,
            current_chunk: 0,
        }
    }

    // called "down" from the collapser

    pub fn collapse_next(&mut self) -> Option<DriverUpdate> {
        let chunk_res = self.chunks.get(self.current_chunk);
        if let Some(chunk) = chunk_res {
            let chunk_clone = chunk.clone(); // TODO - I think this is fine since chunks are lightweight and immutable
            let changed = chunk_clone.collapse_next(self);
            return match changed {
                Some(changes) => Some(self.on_slots_changed(changes)),
                None => self.prepare_next_chunk(),
            };
        }

        None
    }

    // called "up" from chunks

    pub fn get_slot(&mut self, slot_position: Vector3i) -> Option<&Slot> {
        self.slots
            .get(slot_position.y as usize)?
            .get(slot_position.x as usize)?
            .get(slot_position.z as usize)
    }

    pub fn collapse_at(&mut self, slot_position: Vector3i) -> Option<SlotChange> {
        let slot = self.get_slot_mut(slot_position)?;
        slot.collapse(None)
    }

    pub fn constrain_at(&mut self, change: &SlotChange) -> Option<SlotChange> {
        let slot = self.get_slot_mut(change.position)?;
        slot.change(change.new_protos.clone())
    }

    // private

    fn get_slot_mut(&mut self, slot_position: Vector3i) -> Option<&mut Slot> {
        self.slots
            .get_mut(slot_position.y as usize)?
            .get_mut(slot_position.x as usize)?
            .get_mut(slot_position.z as usize)
    }

    fn on_slots_changed(&mut self, changes: Vec<SlotChange>) -> DriverUpdate {
        for change in changes.iter() {
            let pos = change.position;
            let slot = &mut self.slots[pos.y as usize][pos.x as usize][pos.z as usize];
            slot.change(change.new_protos.clone());
        }
        DriverUpdate::new_changes(changes)
    }

    fn prepare_next_chunk(&mut self) -> Option<DriverUpdate> {
        self.current_chunk += 1;
        if self.current_chunk >= self.chunks.len() {
            godot_print!("All chunks processed. Stopping.");
            return Some(DriverUpdate::new_state(CollapserState::STOPPED));
        }

        let next_chunk = self.chunks.get(self.current_chunk);
        let mut overlapping: Vec<Vector3i> = Vec::new();
        let mut neighboring: Vec<Vector3i> = Vec::new();
        match next_chunk {
            None => unreachable!(),
            Some(next) => {
                for i in 0..self.current_chunk {
                    if let Some(other) = self.chunks.get(i) {
                        overlapping.append(&mut next.get_overlapping(other));
                        neighboring.append(&mut next.get_neighbors(other, self.chunk_overlap));
                    }
                }
            }
        }

        let next_chunk = self.chunks[self.current_chunk];
        next_chunk.reset_slots(overlapping, self);
        next_chunk.propagate_from(neighboring, self);
        next_chunk.apply_custom_constraints();

        None
    }
}

fn generate_slots(size: Vector3i, all_protos: Vec<Prototype>) -> Vec<Vec<Vec<Slot>>> {
    let mut slots = vec![];
    for y in 0..size.y {
        let mut plane = vec![];
        for x in 0..size.x {
            let mut row = vec![];
            for z in 0..size.z {
                let slot = Slot::new(Vector3i { x, y, z }, all_protos.clone());
                row.push(slot);
            }
            plane.push(row);
        }
        slots.push(plane);
    }
    slots
}

fn generate_chunks(size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Vec<Chunk> {
    let num_x = ceili((size.x / (chunk_size.x - chunk_overlap)) as f64) as i32;
    let num_y = ceili((size.y / (chunk_size.y - chunk_overlap)) as f64) as i32;
    let num_z = ceili((size.z / (chunk_size.z - chunk_overlap)) as f64) as i32;
    let position_factor = chunk_size - Vector3i::ONE * chunk_overlap;

    let mut chunks = vec![];
    for x in 0..num_x {
        for y in 0..num_y {
            for z in 0..num_z {
                let position = position_factor * Vector3i { x, y, z };
                let new_chunk = Chunk::new(position, chunk_size);
                chunks.push(new_chunk);
            }
        }
    }

    chunks
}
