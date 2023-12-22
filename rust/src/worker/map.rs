use godot::{builtin::Vector3i, engine::utilities::ceili, log::godot_print};

use crate::models::{
    driver_update::{CellChange, DriverUpdate},
    manager::ManagerState,
    prototype::Prototype,
};

use super::{cell::Cell, chunk::Chunk};

pub struct Map {
    pub size: Vector3i,

    cells: Vec<Vec<Vec<Cell>>>,
    chunks: Vec<Chunk>,
    proto_data: Vec<Prototype>,
}

impl Map {
    pub fn new(size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Self {
        let proto_data = Prototype::load();
        godot_print!("Loaded {} prototypes", proto_data.len());
        let cells = generate_cells(size, &proto_data);
        let chunks = generate_chunks(size, chunk_size, chunk_overlap);
        Self {
            size,
            cells,
            chunks,
            proto_data,
        }
    }

    // called "down" from the worker

    pub fn initialize(&mut self) -> Option<DriverUpdate> {
        self.prepare_next_chunk()
    }

    pub fn collapse_next(&mut self, chunk: &Chunk) -> Option<DriverUpdate> {
        let changed = chunk.collapse_next(self);
        return match changed {
            Some(changes) => {
                // godot_print!("chunk collapsed next and got {:?} changes", changes.len());
                Some(self.on_cells_changed(changes))
            }
            None => {
                self.current_chunk += 1;
                self.prepare_next_chunk()
            }
        };
    }

    // called "up" from chunks

    pub fn get_cell(&mut self, cell_position: Vector3i) -> Option<&Cell> {
        self.cells
            .get(cell_position.y as usize)?
            .get(cell_position.x as usize)?
            .get(cell_position.z as usize)
    }

    pub fn get_cell_mut(&mut self, cell_position: Vector3i) -> Option<&mut Cell> {
        self.cells
            .get_mut(cell_position.y as usize)?
            .get_mut(cell_position.x as usize)?
            .get_mut(cell_position.z as usize)
    }

    pub fn reset_cell(&mut self, cell_position: Vector3i) -> Option<CellChange> {
        let all_protos = self.proto_data.clone();
        self.get_cell_mut(cell_position)?.change(&all_protos)
    }

    // private

    fn on_cells_changed(&mut self, changes: Vec<CellChange>) -> DriverUpdate {
        for change in changes.iter() {
            let pos = change.position;
            let cell = &mut self.cells[pos.y as usize][pos.x as usize][pos.z as usize];
            cell.change(&change.new_protos);
        }
        DriverUpdate::new_changes(changes)
    }

    fn prepare_next_chunk(&mut self) -> Option<DriverUpdate> {
        if self.current_chunk >= self.chunks.len() {
            godot_print!("All chunks processed. Stopping.");
            return Some(DriverUpdate::new_state(ManagerState::STOPPED));
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
        for cell in overlapping.iter() {
            if let Some(change) = self.reset_cell(*cell) {
                changes.push(change);
            }
        }

        changes.append(&mut next_chunk.apply_custom_constraints(self));
        changes.append(&mut next_chunk.propagate_all(self));

        changes.append(&mut next_chunk.propagate_from(neighboring, self));
        changes.append(&mut next_chunk.propagate_all(self));

        Some(DriverUpdate::new_changes(changes))
    }
}
