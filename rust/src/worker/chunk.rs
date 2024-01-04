use godot::prelude::*;
use rand::Rng;

use crate::models::driver_update::CellChange;

use super::map::Map;

#[derive(Clone, Copy)]
pub struct Chunk {
    position: Vector3i,
    size: Vector3i,
}

impl Chunk {
    pub fn new(position: Vector3i, size: Vector3i) -> Self {
        Self { position, size }
    }

    // used in tests maybe?
    pub fn _get_all_cells(&self) -> Vec<Vector3i> {
        self.map_filter_cells(|position| Some(position))
    }

    // Used to determine which cells to reset in the initialize chunk phase
    pub fn get_overlapping(&self, other: &Chunk) -> Vec<Vector3i> {
        let self_end = self.position + self.size;
        let other_end = other.position + other.size;

        let start_x = self.position.x.max(other.position.x);
        let end_x = self_end.x.min(other_end.x);

        let start_y = self.position.y.max(other.position.y);
        let end_y = self_end.y.min(other_end.y);

        let start_z = self.position.z.max(other.position.z);
        let end_z = self_end.z.min(other_end.z);

        let mut overlap: Vec<Vector3i> = Vec::new();
        for x in start_x..end_x {
            for y in start_y..end_y {
                for z in start_z..end_z {
                    overlap.push(Vector3i { x, y, z })
                }
            }
        }

        overlap
    }

    // Used to determine which cells to propagate changes in from in the initialize chunk phase
    pub fn get_neighbors(&self, other: &Chunk, n: i32) -> Vec<Vector3i> {
        other.map_filter_cells(|position| {
            if self.contains(position) {
                None
            } else if self.get_cell_neighbors(position, n).len() > 0 {
                Some(position)
            } else {
                None
            }
        })
    }

    // Used in conjunction with get_neighbors to pull in changes from neighboring chunks
    pub fn propagate_from(&self, cells: Vec<Vector3i>, map: &mut Map) -> Vec<CellChange> {
        let mut changes = vec![];
        for cell in cells {
            if let Some(cell) = map.get_cell(cell) {
                changes.append(&mut self.propagate(
                    &CellChange {
                        position: cell.position,
                        new_protos: cell.possibilities.clone(),
                    },
                    map,
                ))
            }
        }

        changes
    }

    pub fn collapse_next(&self, map: &mut Map) -> Option<Vec<CellChange>> {
        let cell_position = self.select_lowest_entropy(map)?;
        let cell = map.get_cell_mut(cell_position)?;
        let change = cell.collapse(None)?;
        Some(self.propagate(&change, map))
    }

    pub fn propagate_all(&self, map: &mut Map) -> Vec<CellChange> {
        self.change_each_cell(map, |position, map| {
            let cell = map.get_cell(position)?;
            let cell_change = CellChange {
                position,
                new_protos: cell.possibilities.clone(),
            };
            Some(self.propagate(&cell_change, map))
        })
    }

    // Propagate a given cell change into other cells within this chunk
    fn propagate(&self, change: &CellChange, map: &mut Map) -> Vec<CellChange> {
        let mut changes: Vec<CellChange> = vec![];
        changes.push(change.clone());

        for neighbor_position in self.get_cell_neighbors(change.position, 1).iter() {
            if let Some(neighbor_cell) = map.get_cell_mut(*neighbor_position) {
                if let Some(neighbor_change) = neighbor_cell.changes_from(change) {
                    if neighbor_change.new_protos.len() == 0 {
                        godot_print!(
                            "overcollapsed {} while propagating {:?}",
                            neighbor_position,
                            change
                        );
                        continue;
                    }

                    neighbor_cell.change(&neighbor_change.new_protos);
                    changes.append(&mut self.propagate(&neighbor_change.clone(), map));
                }
            }
        }

        changes
    }

    // Select the "lowest entropy" cell and collapse it.
    // In reality, there are some rules in place to maintain stability that mean that this is often
    //  not the true lowest-entropy cell.
    fn select_lowest_entropy(&self, map: &mut Map) -> Option<Vector3i> {
        let mut lowest_entropy = usize::MAX;
        let mut lowest_entropy_cells = vec![];

        let start = self.position;
        let end = self.position + self.size;
        for x in start.x..end.x {
            for y in self.position.y..end.y {
                for z in self.position.z..end.z {
                    let position = Vector3i { x, y, z };
                    let cell = map.get_cell(position);
                    if let Some(cell) = cell {
                        let entropy = cell.entropy();
                        if entropy <= 1 || entropy > lowest_entropy {
                            continue;
                        }

                        if entropy < lowest_entropy {
                            lowest_entropy = entropy;
                            lowest_entropy_cells = vec![position];
                        } else if entropy == lowest_entropy {
                            lowest_entropy_cells.push(position);
                        } else {
                            // unreachable!()
                        }
                    }
                }
            }
        }

        if lowest_entropy_cells.len() >= 1 {
            let selected_weight = rand::thread_rng().gen_range(0..lowest_entropy_cells.len());
            return Some(lowest_entropy_cells[selected_weight]);
        }

        None
    }

    // Returns true iff the given position is located within this chunk
    fn contains(&self, position: Vector3i) -> bool {
        let start = self.position;
        let end = self.position + self.size;

        position.x >= start.x
            && position.x < end.x
            && position.y >= start.y
            && position.y < end.y
            && position.z >= start.z
            && position.z < end.z
    }

    // Get all neighboring cells that are exactly one unit away, measured using Manhattan distance
    // That is, only check the 6 cardinal directions directly adjacent to cell_position
    // Diagonal cells are not returned. Cells that are not within this chunk are not returned.
    fn get_cell_neighbors(self, cell_position: Vector3i, n: i32) -> Vec<Vector3i> {
        let mut neighbors = vec![];
        for direction in DIRECTIONS {
            for i in 1..=n {
                let neighbor_position = cell_position + (*direction * i);
                if self.contains(neighbor_position) {
                    neighbors.push(neighbor_position);
                }
            }
        }

        neighbors
    }

    // ITERATING UTILS

    fn change_each_cell<F: Fn(Vector3i, &mut Map) -> Option<Vec<CellChange>>>(
        &self,
        map: &mut Map,
        f: F,
    ) -> Vec<CellChange> {
        let mut changes = vec![];

        let start = self.position;
        let end = self.position + self.size;
        for x in start.x..end.x {
            for y in start.y..end.y {
                for z in start.z..end.z {
                    if let Some(mut changes_applied) = f(Vector3i { x, y, z }, map) {
                        changes.append(&mut changes_applied)
                    }
                }
            }
        }

        changes
    }

    fn map_filter_cells<F: Fn(Vector3i) -> Option<Vector3i>>(&self, f: F) -> Vec<Vector3i> {
        let mut cells = vec![];

        let start = self.position;
        let end = self.position + self.size;
        for x in start.x..end.x {
            for y in start.y..end.y {
                for z in start.z..end.z {
                    if let Some(position) = f(Vector3i { x, y, z }) {
                        cells.push(position);
                    }
                }
            }
        }

        cells
    }
}

const DIRECTIONS: &'static [Vector3i] = &[
    Vector3i::UP,
    Vector3i::DOWN,
    Vector3i::RIGHT,
    Vector3i::LEFT,
    Vector3i::FORWARD,
    Vector3i::BACK,
];
