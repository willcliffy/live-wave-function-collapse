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
    pub size: Vector3i,
    slots: Vec<Vec<Vec<Slot>>>,
    chunks: Vec<Chunk>,
    chunk_overlap: i32,
    current_chunk: usize,
    proto_data: Vec<Prototype>,
}

impl Map {
    pub fn new(size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Self {
        let proto_data = Prototype::load();
        godot_print!("Loaded {} prototypes", proto_data.len());
        let slots = generate_slots(size, &proto_data);
        let chunks = generate_chunks(size, chunk_size, chunk_overlap);
        Self {
            size,
            slots,
            chunks,
            chunk_overlap,
            current_chunk: 0,
            proto_data,
        }
    }

    // called "down" from the collapser

    pub fn initialize(&mut self) -> Option<DriverUpdate> {
        self.prepare_next_chunk()
    }

    pub fn collapse_next(&mut self) -> Option<DriverUpdate> {
        let chunk_res = self.chunks.get(self.current_chunk);
        if let Some(chunk) = chunk_res {
            let changed = chunk.clone().collapse_next(self);
            return match changed {
                Some(changes) => {
                    // godot_print!("chunk collapsed next and got {:?} changes", changes.len());
                    Some(self.on_slots_changed(changes))
                }
                None => {
                    self.current_chunk += 1;
                    self.prepare_next_chunk()
                }
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

    pub fn get_slot_mut(&mut self, slot_position: Vector3i) -> Option<&mut Slot> {
        self.slots
            .get_mut(slot_position.y as usize)?
            .get_mut(slot_position.x as usize)?
            .get_mut(slot_position.z as usize)
    }

    pub fn reset_slot(&mut self, slot_position: Vector3i) -> Option<SlotChange> {
        let all_protos = self.proto_data.clone();
        self.get_slot_mut(slot_position)?.change(&all_protos)
    }

    // private

    fn on_slots_changed(&mut self, changes: Vec<SlotChange>) -> DriverUpdate {
        for change in changes.iter() {
            let pos = change.position;
            let slot = &mut self.slots[pos.y as usize][pos.x as usize][pos.z as usize];
            slot.change(&change.new_protos);
        }
        DriverUpdate::new_changes(changes)
    }

    fn prepare_next_chunk(&mut self) -> Option<DriverUpdate> {
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

        let mut changes = vec![];

        let next_chunk = self.chunks[self.current_chunk];
        for slot in overlapping.iter() {
            if let Some(change) = self.reset_slot(*slot) {
                changes.push(change);
            }
        }

        next_chunk.apply_custom_constraints(self);

        changes.append(&mut next_chunk.propagate_from(neighboring, self));

        next_chunk.propagate_all(self);

        Some(DriverUpdate::new_changes(changes))
    }
}

fn generate_slots(size: Vector3i, all_protos: &Vec<Prototype>) -> Vec<Vec<Vec<Slot>>> {
    // let uncapped_x_min = Prototype::uncapped(all_protos, Vector3i::LEFT);
    // let uncapped_x_max = Prototype::uncapped(all_protos, Vector3i::RIGHT);
    // let uncapped_y_min = Prototype::uncapped(all_protos, Vector3i::DOWN);
    // let uncapped_y_max = Prototype::uncapped(all_protos, Vector3i::UP);
    // let uncapped_z_min = Prototype::uncapped(all_protos, Vector3i::FORWARD);
    // let uncapped_z_max = Prototype::uncapped(all_protos, Vector3i::BACK);
    // let not_bot = Prototype::not_constrained(all_protos, "BOT".into());

    let mut slots = vec![];
    for y in 0..size.y {
        let mut plane = vec![];
        for x in 0..size.x {
            let mut row = vec![];
            for z in 0..size.z {
                let mut slot_protos = all_protos.clone();

                if x == 0 {
                    Prototype::retain_uncapped(&mut slot_protos, Vector3i::LEFT);
                } else if x == size.x - 1 {
                    Prototype::retain_uncapped(&mut slot_protos, Vector3i::RIGHT);
                }

                if y == 0 {
                    Prototype::retain_uncapped(&mut slot_protos, Vector3i::DOWN);
                } else {
                    Prototype::retain_not_constrained(&mut slot_protos, "BOT".into());
                    if y == size.y - 1 {
                        Prototype::retain_uncapped(&mut slot_protos, Vector3i::UP);
                    }
                }

                if z == 0 {
                    Prototype::retain_uncapped(&mut slot_protos, Vector3i::FORWARD);
                } else if z == size.z - 1 {
                    Prototype::retain_uncapped(&mut slot_protos, Vector3i::BACK);
                }

                let slot = Slot::new(Vector3i { x, y, z }, slot_protos);
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
    for y in 0..num_y {
        for x in 0..num_x {
            for z in 0..num_z {
                let position = position_factor * Vector3i { x, y, z };
                let new_chunk = Chunk::new(position, chunk_size);
                chunks.push(new_chunk);
            }
        }
    }

    chunks
}
