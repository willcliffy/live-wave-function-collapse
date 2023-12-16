use std::cmp::min;

use godot::prelude::*;
use rand::Rng;

use crate::{
    map::Map,
    models::{driver_update::SlotChange, prototype::Prototype},
    slot::Slot,
};

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

    pub fn propagate_from(&self, slots: Vec<Vector3i>, map: &mut Map) -> Vec<SlotChange> {
        //unreachable!();
        let mut changes = vec![];
        for slot in slots {
            if let Some(slot) = map.get_slot(slot) {
                changes.append(&mut self.propagate(
                    &SlotChange {
                        position: slot.position,
                        new_protos: slot.possibilities.clone(),
                    },
                    map,
                ))
            }
        }

        changes
    }

    pub fn collapse_next(&self, map: &mut Map) -> Option<Vec<SlotChange>> {
        if let Some(slot_position) = self.select_lowest_entropy(map) {
            if let Some(slot) = map.get_slot_mut(slot_position) {
                let change = slot.collapse(None);
                return match change {
                    Some(change) => Some(self.propagate(&change, map)),
                    None => {
                        godot_print!("failed to collapse at {}", slot_position);
                        None
                    }
                };
            }
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
                        let mut entropy = slot.entropy();
                        if entropy <= 1 || entropy > lowest_entropy {
                            continue;
                        }

                        // TODO - apply custom entropy rules here
                        // In the GDScript implementation, I added 1 along the bounding box of the
                        // chunk, 2 at the top of the chunk, and added y to all cells' entropy
                        if y == 0 {
                            entropy += 100;
                        } else {
                            entropy += y as usize;
                        }

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

    fn within(&self, position: Vector3i) -> bool {
        let start = self.position;
        let end = self.position + self.size;

        position.x >= start.x
            && position.x < end.x
            && position.y >= start.y
            && position.y < end.y
            && position.z >= start.z
            && position.z < end.z
    }

    fn get_slot_neighbors(self, change: &SlotChange, _map: &mut Map) -> Vec<Vector3i> {
        let mut neighbors = vec![];
        for direction in DIRECTIONS {
            let neighbor_position = change.position + *direction;
            if self.within(neighbor_position) {
                neighbors.push(neighbor_position);
            }
        }

        neighbors
    }

    // pub fn changes_from(&self, other: &SlotChange) -> Option<SlotChange> {
    //     let mut new_protos = vec![];
    //     let direction = other.position - self.position;

    //     for proto in self.possibilities.iter() {
    //         if proto.compatible_with_any(other.new_protos.clone(), direction) {
    //             new_protos.push(proto.clone())
    //         }
    //     }

    //     if new_protos.len() != self.possibilities.len() {
    //         return Some(SlotChange {
    //             position: self.position,
    //             new_protos,
    //         });
    //     }

    //     None
    // }

    pub fn apply_custom_constraints(&self, map: &mut Map) {
        let map_size = map.size;
        let chunk_top_y = min(self.position.y + self.size.y, map_size.y) - 1;

        for x in self.position.x..self.position.x + self.size.x {
            for y in self.position.y..self.position.y + self.size.y {
                for z in self.position.z..self.position.z + self.size.z {
                    let position = Vector3i { x, y, z };
                    let slot = map.get_slot_mut(position);
                    if let Some(slot) = slot {
                        if position.y == 0 {
                            Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::DOWN);
                        } else {
                            Prototype::retain_not_constrained(
                                &mut slot.possibilities,
                                "BOT".into(),
                            );
                        }

                        if position.y == chunk_top_y {
                            Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::UP);
                        }

                        if position.x == 0 {
                            Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::LEFT);
                        }

                        if position.x == map_size.x - 1 {
                            Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::RIGHT);
                        }

                        if position.z == 0 {
                            Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::FORWARD);
                        }

                        if position.z == map_size.z - 1 {
                            Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::BACK);
                        }
                    }
                }
            }
        }

        return;
    }

    fn calculate_changes(self, source: &SlotChange, target: &Slot) -> Option<SlotChange> {
        let direction = source.position - target.position;
        let mut new_protos = vec![];
        for proto in target.possibilities.iter() {
            if proto.compatible_with_any(&source.new_protos, direction) {
                new_protos.push(proto.clone());
            }
        }

        if new_protos.len() != target.possibilities.len() {
            return Some(SlotChange {
                position: target.position,
                new_protos,
            });
        }

        None
    }

    fn propagate(&self, change: &SlotChange, map: &mut Map) -> Vec<SlotChange> {
        let mut changes: Vec<SlotChange> = vec![];
        changes.push(change.clone());

        // godot_print!("neighbors: {:?}", neighbors);
        for neighbor in self.get_slot_neighbors(change, map).iter() {
            if let Some(neighbor_slot) = map.get_slot_mut(*neighbor) {
                if let Some(neighbor_change) = self.calculate_changes(change, neighbor_slot) {
                    neighbor_slot.change(&neighbor_change.new_protos);
                    changes.append(&mut self.propagate(&neighbor_change.clone(), map));
                    changes.push(neighbor_change);
                }
            }
        }

        changes
    }

    pub fn propagate_all(&self, map: &mut Map) -> Vec<SlotChange> {
        let mut changes = vec![];

        for x in self.position.x..self.position.x + self.size.x {
            for y in self.position.y..self.position.y + self.size.y {
                for z in self.position.z..self.position.z + self.size.z {
                    let position = Vector3i { x, y, z };
                    let slot = map.get_slot_mut(position);
                    if let Some(slot) = slot {
                        changes.append(&mut self.propagate(
                            &SlotChange {
                                position: position,
                                new_protos: slot.possibilities.clone(),
                            },
                            map,
                        ))
                    }
                }
            }
        }

        changes
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
