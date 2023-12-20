use godot::{builtin::Vector3i, engine::utilities::ceili, log::godot_print};

use crate::models::{
    collapser_state::CollapserState,
    driver_update::{CellChange, DriverUpdate},
    prototype::Prototype,
};

use super::{cell::Cell, chunk::Chunk};

pub struct Map {
    pub size: Vector3i,
    cells: Vec<Vec<Vec<Cell>>>,
    chunks: Vec<Chunk>,
    chunk_overlap: i32,
    current_chunk: usize,
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
        let chunk = self.chunks.get(self.current_chunk)?;
        let changed = chunk.clone().collapse_next(self);
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

fn generate_cells(size: Vector3i, all_protos: &Vec<Prototype>) -> Vec<Vec<Vec<Cell>>> {
    // let uncapped_x_min = Prototype::uncapped(all_protos, Vector3i::LEFT);
    // let uncapped_x_max = Prototype::uncapped(all_protos, Vector3i::RIGHT);
    // let uncapped_y_min = Prototype::uncapped(all_protos, Vector3i::DOWN);
    // let uncapped_y_max = Prototype::uncapped(all_protos, Vector3i::UP);
    // let uncapped_z_min = Prototype::uncapped(all_protos, Vector3i::FORWARD);
    // let uncapped_z_max = Prototype::uncapped(all_protos, Vector3i::BACK);
    // let not_bot = Prototype::not_constrained(all_protos, "BOT".into());

    let mut cells = vec![];
    for y in 0..size.y {
        let mut plane = vec![];
        for x in 0..size.x {
            let mut row = vec![];
            for z in 0..size.z {
                let mut cell_protos = all_protos.clone();

                if x == 0 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::LEFT);
                } else if x == size.x - 1 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::RIGHT);
                }

                if y == 0 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::DOWN);
                } else {
                    Prototype::retain_not_constrained(&mut cell_protos, "BOT".into());
                    if y == size.y - 1 {
                        Prototype::retain_uncapped(&mut cell_protos, Vector3i::UP);
                    }
                }

                if z == 0 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::FORWARD);
                } else if z == size.z - 1 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::BACK);
                }

                let cell = Cell::new(Vector3i { x, y, z }, cell_protos);
                row.push(cell);
            }
            plane.push(row);
        }
        cells.push(plane);
    }
    cells
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
