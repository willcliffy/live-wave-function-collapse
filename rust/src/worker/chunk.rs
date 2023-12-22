use std::cmp::min;

use godot::prelude::*;
use rand::Rng;

use crate::models::prototype::Prototype;

use super::cell::Cell;

#[derive(Clone, Copy)]
pub struct Chunk {
    position: Vector3i,
    size: Vector3i,
}

impl Chunk {
    pub fn new(position: Vector3i, size: Vector3i) -> Self {
        Self { position, size }
    }

    pub fn bounds(&self) -> (Vector3i, Vector3i) {
        (self.position, self.position + self.size)
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
    pub fn propagate_from(&self, cells: &mut Vec<Cell>, other_cells: Vec<Cell>) -> Vec<Cell> {
        let mut changes = vec![];
        for cell in other_cells.iter() {
            changes.append(&mut self.propagate(&cell, cells))
        }

        changes
    }

    // Choose a cell contained within this chunk and collapse it
    pub fn collapse_next(&self, cells: &mut Vec<Cell>) -> Option<Vec<Cell>> {
        let cell_position = self.select_lowest_entropy(&cells)?;
        let cell_index = self.get_index(cell_position);
        let cell = cells.get_mut(cell_index)?;
        cell.collapse(None)?;
        Some(self.propagate(&cell.clone(), cells))
    }

    // No uncapped cells along the edge of the map. No uncapped cells along the top of the chunk
    // Prototypes marked `"constrain_to": "BOT"` should only appear in cells where y = 0
    pub fn apply_custom_constraints(&self, cells: &mut Vec<Cell>, map_size: Vector3i) {
        let chunk_top_y = min(self.position.y + self.size.y, map_size.y) - 1;
        for cell in cells.iter_mut() {
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
    }

    // Should not be necessary theoretically, but useful in many situations and as part of several
    //  strategies to maintain stability
    // pub fn propagate_all(&self, cells: Vec<Cell>) -> Vec<Cell> {
    //     let mut changes = vec![];
    //     for cell in cells.iter() {
    //         changes.append(&mut self.propagate(cell, cells));
    //     }

    //     changes
    // }

    // Propagate a given cell change into other cells within this chunk
    fn propagate(&self, changed: &Cell, cells: &mut Vec<Cell>) -> Vec<Cell> {
        let mut changes: Vec<Cell> = vec![];
        changes.push(changed.clone());

        for neighbor_position in self.get_cell_neighbors(changed.position, 1).iter() {
            let neighbor_index = self.get_index(*neighbor_position);
            let neighbor_cell = cells.get(neighbor_index);
            match neighbor_cell {
                Some(neighbor) => {
                    if let Some(neighbor_changed) = neighbor.changes_from(changed) {
                        cells[neighbor_index] = neighbor_changed.clone();
                        changes.append(&mut self.propagate(&neighbor_changed, cells));
                    }
                }
                None => {
                    godot_print!("Failed to get cell in propagate: index {} (for position {}) is out of bounds {}",
                        neighbor_index,
                        neighbor_position,
                        cells.len())
                }
            }
        }

        changes
    }

    // Select the "lowest entropy" cell and collapse it.
    // In reality, there are some rules in place to maintain stability that mean that this is often
    //  not the true lowest-entropy cell.
    fn select_lowest_entropy(&self, cells: &Vec<Cell>) -> Option<Vector3i> {
        let mut lowest_entropy = usize::MAX;
        let mut lowest_entropy_cells = vec![];

        for cell in cells.iter() {
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

    // TODO: repeated in library.rs
    fn get_index(&self, location: Vector3i) -> usize {
        ((location.y - self.position.y) * (self.size.x * self.size.z)
            + (location.x - self.position.x) * self.size.z
            + (location.z - self.position.z)) as usize
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
