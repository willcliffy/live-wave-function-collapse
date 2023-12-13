use godot::{
    builtin::{Vector3, Vector3i},
    engine::utilities::ceili,
    log::godot_print,
};

use crate::{
    chunk::Chunk,
    models::{
        collapser_state::CollapserState,
        driver_update::{DriverUpdate, SlotChange},
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
    pub fn new(size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Self {
        let mut slots = vec![];
        for y in 0..size.y as i32 {
            let mut plane = vec![];
            for x in 0..size.x as i32 {
                let mut row = vec![];
                for z in 0..size.z as i32 {
                    let slot = Slot::new(Vector3 {
                        x: x as f32,
                        y: y as f32,
                        z: z as f32,
                    });
                    row.push(slot);
                }
                plane.push(row);
            }
            slots.push(plane);
        }

        let num_x = ceili((size.x / (chunk_size.x - chunk_overlap)) as f64);
        let num_y = ceili((size.y / (chunk_size.y - chunk_overlap)) as f64);
        let num_z = ceili((size.z / (chunk_size.z - chunk_overlap)) as f64);
        let position_factor = chunk_size - Vector3i::ONE * chunk_overlap;

        let mut chunks = vec![];
        for x_chunk in 0..num_x {
            for y_chunk in 0..num_y {
                for z_chunk in 0..num_z {
                    let position = position_factor
                        * Vector3i {
                            x: x_chunk as i32,
                            y: y_chunk as i32,
                            z: z_chunk as i32,
                        };
                    let new_chunk = Chunk::new(position, chunk_size);
                    chunks.push(new_chunk);
                }
            }
        }

        Self {
            slots,
            chunks,
            chunk_overlap,
            current_chunk: 0,
        }
    }

    pub fn collapse_next(&mut self) -> Option<DriverUpdate> {
        let chunk = self.chunks.get(self.current_chunk);
        if let Some(chunk) = chunk {
            let changed = chunk.clone().collapse_next(self);
            return match changed {
                Some(changes) => Some(self.on_slots_changed(changes)),
                None => self.prepare_next_chunk(),
            };
        }

        None
    }

    fn on_slots_changed(&mut self, changes: Vec<SlotChange>) -> DriverUpdate {
        for change in changes.iter() {
            let pos = change.position;
            let slot = &mut self.slots[pos.y as usize][pos.x as usize][pos.z as usize];
            slot.changed(change.new_protos.clone());
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
