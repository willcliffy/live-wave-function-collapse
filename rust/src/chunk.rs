use godot::prelude::*;
use rand::Rng;

use crate::{map::Map, models::driver_update::SlotChange};

#[derive(Clone, Copy)]
pub struct Chunk {
    position: Vector3i,
    size: Vector3i,
}

impl Chunk {
    pub fn new(position: Vector3i, size: Vector3i) -> Self {
        Self { position, size }
    }

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
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                for z in start_z..=end_z {
                    overlap.push(Vector3i { x, y, z })
                }
            }
        }

        overlap
    }

    pub fn get_neighbors(&self, other: &Chunk, n: i32) -> Vec<Vector3i> {
        let self_end = self.position + self.size;
        let other_end = other.position + other.size;

        let mut neighbors: Vec<Vector3i> = Vec::new();

        for x in other.position.x..=other_end.x {
            for y in other.position.y..=other_end.y {
                for z in other.position.z..=other_end.z {
                    let distance_x = if x < self.position.x {
                        self.position.x - x
                    } else if x > self_end.x {
                        x - self_end.x
                    } else {
                        0
                    };
                    let distance_y = if y < self.position.y {
                        self.position.y - y
                    } else if y > self_end.y {
                        y - self_end.y
                    } else {
                        0
                    };
                    let distance_z = if z < self.position.z {
                        self.position.z - z
                    } else if z > self_end.z {
                        z - self_end.z
                    } else {
                        0
                    };

                    let total_distance = distance_x.max(0) + distance_y.max(0) + distance_z.max(0);
                    if total_distance <= n
                        && (x < self.position.x
                            || x > self_end.x
                            || y < self.position.y
                            || y > self_end.y
                            || z < self.position.z
                            || z > self_end.z)
                    {
                        neighbors.push(Vector3i { x, y, z });
                    }
                }
            }
        }

        neighbors
    }

    pub fn reset_slots(&self, _slots: Vec<Vector3i>, _map: &mut Map) {
        //unreachable!();
        godot_print!("reset_slots NOT YET IMPLEMENTED");
    }

    pub fn propagate_from(&self, _slots: Vec<Vector3i>, _map: &mut Map) {
        //unreachable!();
        godot_print!("propagate_from NOT YET IMPLEMENTED");
    }

    pub fn apply_custom_constraints(&self) {
        // unreachable!();
        godot_print!("apply_custom_constraints NOT YET IMPLEMENTED");
        return;
    }

    pub fn collapse_next(&self, map: &mut Map) -> Option<Vec<SlotChange>> {
        if let Some(slot_position) = self.select_lowest_entropy(map) {
            let change = map.collapse_at(slot_position);
            return match change {
                Some(change) => Some(self.propagate(&change, map)),
                None => {
                    godot_print!("failed to collapse at {}", slot_position);
                    None
                }
            };
        }

        None
    }

    fn select_lowest_entropy(&self, map: &mut Map) -> Option<Vector3i> {
        let mut lowest_entropy = usize::MAX;
        let mut lowest_entropy_slots = vec![];
        for x in self.position.x..self.position.x + self.size.x {
            for y in self.position.y..self.position.y + self.size.y {
                for z in self.position.z..self.position.z + self.size.z {
                    let position = Vector3i { x, y, z };
                    let slot = map.get_slot(position);
                    if let Some(slot) = slot {
                        let entropy = slot.entropy();
                        if entropy <= 1 || entropy > lowest_entropy {
                            continue;
                        }

                        // TODO - apply custom entropy rules here
                        // In the GDScript implementation, I added 1 along the bounding box of the
                        // chunk, 2 at the top of the chunk, and added y to all cells' entropy
                        if entropy < lowest_entropy {
                            lowest_entropy = entropy;
                            lowest_entropy_slots = vec![position];
                        } else if entropy == lowest_entropy {
                            lowest_entropy_slots.push(position);
                        } else {
                            // unreachable!()
                        }
                    }
                }
            }
        }

        if lowest_entropy_slots.len() >= 1 {
            let selected_weight = rand::thread_rng().gen_range(0..lowest_entropy_slots.len());
            return Some(lowest_entropy_slots[selected_weight]);
        }

        None
    }

    fn get_slot_neighbors(self, _map: &mut Map) -> Vec<Vector3i> {
        vec![]
    }

    fn propagate(&self, change: &SlotChange, map: &mut Map) -> Vec<SlotChange> {
        self._propagate_recursive(change, map, 0)
    }

    fn _propagate_recursive(
        &self,
        change: &SlotChange,
        map: &mut Map,
        _depth: i64,
    ) -> Vec<SlotChange> {
        let change_copy = change.clone();
        let mut changes = vec![change_copy];

        let neighbors = self.get_slot_neighbors(map);
        for neighbor in neighbors.iter() {
            let neighbor_slot_op = map.get_slot(*neighbor);
            match neighbor_slot_op {
                Some(neighbor_slot) => match neighbor_slot.changes_from(change) {
                    Some(neighbor_change) => {
                        map.constrain_at(&neighbor_change);
                        changes.push(neighbor_change)
                    }
                    None => continue,
                },
                None => godot_print!("Tried to get neighbor that doesn't exist!"),
            }
        }

        changes
    }
}
