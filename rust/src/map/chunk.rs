use std::cmp::min;

use godot::prelude::*;
use rand::Rng;

use crate::models::{library::Range, prototype::Prototype};

use crate::map::models::ChunkState;
use crate::worker::models::WorkerUpdateStatus;

use super::cell::Cell;

#[derive(Clone)]
pub struct Chunk {
    pub state: ChunkState,
    pub position: Vector3i,
    pub size: Vector3i,
}

impl Chunk {
    pub fn new(position: Vector3i, size: Vector3i) -> Self {
        let state = ChunkState::Ready;
        Self {
            state,
            position,
            size,
        }
    }

    pub fn bounds(&self) -> (Vector3i, Vector3i) {
        (self.position, self.position + self.size)
    }

    pub fn is_overlapping(&self, other: &Chunk) -> bool {
        let (self_start, self_end) = self.bounds();
        let (other_start, other_end) = other.bounds();
        let overlap_x = self_end.x >= other_start.x && self_start.x <= other_end.x;
        let overlap_y = self_end.y >= other_start.y && self_start.y <= other_end.y;
        let overlap_z = self_end.z >= other_start.z && self_start.z <= other_end.z;

        overlap_x && overlap_y && overlap_z
    }

    pub fn reset_cells(
        &self,
        range: &mut Range<Cell>,
        proto_data: &Vec<Prototype>,
        map_size: Vector3i,
    ) -> anyhow::Result<Vec<Cell>> {
        for cell in &mut range.books {
            let mut cell_protos = proto_data.clone();

            if cell.position.x == 0 {
                Prototype::retain_uncapped(&mut cell_protos, Vector3i::LEFT);
            } else if cell.position.x == map_size.x - 1 {
                Prototype::retain_uncapped(&mut cell_protos, Vector3i::RIGHT);
            }

            if cell.position.y == 0 {
                Prototype::retain_uncapped(&mut cell_protos, Vector3i::DOWN);
            } else {
                Prototype::retain_not_constrained(&mut cell_protos, "BOT".into());
                if cell.position.y == map_size.y - 1 {
                    Prototype::retain_uncapped(&mut cell_protos, Vector3i::UP);
                }
            }

            if cell.position.z == 0 {
                Prototype::retain_uncapped(&mut cell_protos, Vector3i::FORWARD);
            } else if cell.position.z == map_size.z - 1 {
                Prototype::retain_uncapped(&mut cell_protos, Vector3i::BACK);
            }
            cell.change(&cell_protos);
        }

        let cells_clone = range.books.clone();
        Ok(cells_clone)
    }

    // Choose a cell contained within this chunk and collapse it
    pub fn collapse_next(&mut self, range: &mut Range<Cell>) -> anyhow::Result<WorkerUpdateStatus> {
        let result = match self.select_lowest_entropy(&range.books) {
            Some(cell_position) => {
                let cell_index = range.index(cell_position);
                if cell_position != range.books[cell_index].position {
                    godot_print!(
                        "[C] WARNING: Cell position mismatch - check index implementation {} (index {}) reported position {}",
                        cell_position,
                        cell_index,
                        range.books[cell_index].position,
                    );
                }

                match self.collapse_cell(range, cell_index) {
                    Some(cell) => {
                        let changes = self.propagate(&cell, range)?;
                        Ok(WorkerUpdateStatus::Ok(changes))
                    }
                    None => Err(anyhow::anyhow!("Failed to collapse next!")),
                }
            }
            None => {
                self.state = ChunkState::Collapsed;
                Ok(WorkerUpdateStatus::Done)
            }
        };

        result
    }

    fn collapse_cell(&self, range: &mut Range<Cell>, collapse_index: usize) -> Option<Cell> {
        let cell = range.books.get(collapse_index)?;
        let collapsed = cell.collapsed(None)?;
        let collapsed_clone = collapsed.clone();
        range.books[collapse_index] = collapsed;
        Some(collapsed_clone)
    }

    // No uncapped cells along the edge of the map. No uncapped cells along the top of the chunk
    // Prototypes marked `"constrain_to": "BOT"` should only appear in cells where y = 0
    pub fn apply_constraints(
        &self,
        range: &mut Range<Cell>,
        map_size: Vector3i,
    ) -> anyhow::Result<Vec<Cell>> {
        let chunk_top_y = min(self.position.y + self.size.y, map_size.y) - 1;
        for cell in range.books.iter_mut() {
            if cell.position.y == 0 {
                Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::DOWN);
            } else {
                Prototype::retain_not_constrained(&mut cell.possibilities, "BOT".into());
            }

            if cell.position.y == chunk_top_y {
                Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::UP);
            }

            if cell.position.x == 0 {
                Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::LEFT);
            }

            if cell.position.x == map_size.x - 1 {
                Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::RIGHT);
            }

            if cell.position.z == 0 {
                Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::FORWARD);
            }

            if cell.position.z == map_size.z - 1 {
                Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::BACK);
            }
        }

        Ok(vec![])
    }

    // Should not be necessary theoretically, but useful in many situations and as part of several
    //  strategies to maintain stability
    pub fn propagate_all(&self, range: &mut Range<Cell>) -> anyhow::Result<Vec<Cell>> {
        let mut changes = vec![];
        for i in 0..range.books.len() {
            let inner_changes = &mut self.propagate(&range.books[i].clone(), range)?;
            changes.append(inner_changes);
        }

        Ok(changes)
    }

    // Propagate a given cell change into other cells within this chunk
    fn propagate(&self, changed: &Cell, range: &mut Range<Cell>) -> anyhow::Result<Vec<Cell>> {
        let mut changes: Vec<Cell> = vec![];
        changes.push(changed.clone());

        for neighbor_position in range.get_neighbors(changed.position).iter() {
            let neighbor_index = range.index(*neighbor_position);
            let neighbor_cell = range.books.get(neighbor_index);
            match neighbor_cell {
                None => continue,
                Some(neighbor) => {
                    if *neighbor_position != neighbor.position {
                        godot_print!(
                            "[C] WARNING: Cell position mismatch - check index implementation {} (index {}) reported position {}. Range: {}",
                            neighbor_position,
                            neighbor_index,
                            neighbor.position,
                            range.size,
                        );
                    }
                    if let Some(neighbor_changed) = neighbor.changes_from(changed) {
                        if neighbor_changed.possibilities.len() == 0 {
                            return Err(anyhow::anyhow!(
                                "Overcollapsed: neighbor {} had {:?}, collapsed because {} has {:?}",
                                neighbor.position,
                                neighbor
                                    .possibilities
                                    .iter()
                                    .map(|p| p.id.clone())
                                    .collect::<Vec<String>>(),
                                changed.position,
                                changed
                                    .possibilities
                                    .iter()
                                    .map(|p| p.id.clone())
                                    .collect::<Vec<String>>()
                            ));
                        } else {
                            let neighbor_changed_clone = neighbor_changed.clone();
                            range.books[neighbor_index] = neighbor_changed;
                            let inner_changes =
                                &mut self.propagate(&neighbor_changed_clone, range)?;
                            changes.append(inner_changes);
                        }
                    }
                }
            }
        }

        Ok(changes)
    }

    // Select the "lowest entropy" cell and collapse it.
    // In reality, there are some rules in place to maintain stability that mean that this is often
    //  not the true lowest-entropy cell.
    fn select_lowest_entropy(&self, cells: &Vec<Cell>) -> Option<Vector3i> {
        let mut lowest_entropy = usize::MAX;
        let mut lowest_entropy_cells = vec![];

        for cell in cells {
            let mut entropy = cell.entropy();
            if entropy <= 1 || entropy > lowest_entropy {
                continue;
            }

            // TODO - apply custom entropy rules here
            // In the GDScript implementation, I added 1 along the bounding box of the
            // chunk, 2 at the top of the chunk, and added y to all cells' entropy
            if cell.position.y == 0 {
                entropy += 100;
            }

            if entropy < lowest_entropy {
                lowest_entropy = entropy;
                lowest_entropy_cells = vec![cell.position];
            } else if entropy == lowest_entropy {
                lowest_entropy_cells.push(cell.position);
            } else {
                // TODO - this is reachable since we added custom entropy rules
                // need to think about what to do here.
                // unreachable!()
            }
        }

        if lowest_entropy_cells.len() >= 1 {
            let selected_weight = rand::thread_rng().gen_range(0..lowest_entropy_cells.len());
            return Some(lowest_entropy_cells[selected_weight]);
        }

        None
    }
}
