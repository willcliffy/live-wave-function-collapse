use std::cmp::min;

use godot::prelude::*;
use rand::Rng;

use crate::models::{driver_update::SlotChange, prototype::Prototype};

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

    pub fn _get_all_slots(&self) -> Vec<Vector3i> {
        self.map_filter_slots(|position| Some(position))
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
        for x in start_x..end_x {
            for y in start_y..end_y {
                for z in start_z..end_z {
                    overlap.push(Vector3i { x, y, z })
                }
            }
        }

        overlap
    }

    pub fn get_neighbors(&self, other: &Chunk, n: i32) -> Vec<Vector3i> {
        other.map_filter_slots(|position| {
            if self.contains(position) {
                println!(
                    "returning none since {} is contained in this chunk",
                    position
                );
                None
            } else if self.get_slot_neighbors(position, n).len() > 0 {
                println!(
                    "returning Some since {} has neighbors in this chunk",
                    position
                );
                Some(position)
            } else {
                println!(
                    "returning none since {} has no neighbors in this chunk",
                    position
                );
                None
            }
        })
    }

    pub fn propagate_from(&self, slots: Vec<Vector3i>, map: &mut Map) -> Vec<SlotChange> {
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

        let start = self.position;
        let end = self.position + self.size;
        for x in start.x..end.x {
            for y in self.position.y..end.y {
                for z in self.position.z..end.z {
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
                            unreachable!()
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

    fn get_slot_neighbors(self, slot_position: Vector3i, n: i32) -> Vec<Vector3i> {
        let mut neighbors = vec![];
        for direction in DIRECTIONS {
            for i in 1..=n {
                let neighbor_position = slot_position + (*direction * i);
                if self.contains(neighbor_position) {
                    neighbors.push(neighbor_position);
                }
            }
        }

        neighbors
    }

    pub fn apply_custom_constraints(&self, map: &mut Map) -> Vec<SlotChange> {
        let chunk_top_y = min(self.position.y + self.size.y, map.size.y) - 1;
        self.change_each_slot(map, |position, map| {
            let map_size = map.size;
            let slot = map.get_slot_mut(position);
            if let Some(slot) = slot {
                let old_entropy = slot.possibilities.len();

                if position.y == 0 {
                    Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::DOWN);
                } else {
                    Prototype::retain_not_constrained(&mut slot.possibilities, "BOT".into());
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

                if slot.possibilities.len() != old_entropy {
                    return vec![SlotChange {
                        position,
                        new_protos: slot.possibilities.clone(),
                    }];
                }
            }

            vec![]
        })

        // for x in start.x..end.x {
        //     for y in start.y..end.y {
        //         for z in start.z..end.z {
        //             let position = Vector3i { x, y, z };
        //             let slot = map.get_slot_mut(position);
        //             if let Some(slot) = slot {
        //                 let old_entropy = slot.possibilities.len();
        //                 if position.y == 0 {
        //                     Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::DOWN);
        //                 } else {
        //                     Prototype::retain_not_constrained(
        //                         &mut slot.possibilities,
        //                         "BOT".into(),
        //                     );
        //                 }

        //                 if position.y == chunk_top_y {
        //                     Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::UP);
        //                 }

        //                 if position.x == 0 {
        //                     Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::LEFT);
        //                 }

        //                 if position.x == map_size.x - 1 {
        //                     Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::RIGHT);
        //                 }

        //                 if position.z == 0 {
        //                     Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::FORWARD);
        //                 }

        //                 if position.z == map_size.z - 1 {
        //                     Prototype::retain_uncapped(&mut slot.possibilities, Vector3i::BACK);
        //                 }

        //                 if slot.possibilities.len() != old_entropy {
        //                     changes.push(SlotChange {
        //                         position,
        //                         new_protos: slot.possibilities.clone(),
        //                     })
        //                 }
        //             }
        //         }
        //     }
        // }

        // changes
    }

    fn propagate(&self, change: &SlotChange, map: &mut Map) -> Vec<SlotChange> {
        let mut changes: Vec<SlotChange> = vec![];
        changes.push(change.clone());

        for neighbor in self.get_slot_neighbors(change.position, 1).iter() {
            if let Some(neighbor_slot) = map.get_slot_mut(*neighbor) {
                if let Some(neighbor_change) = neighbor_slot.changes_from(change) {
                    neighbor_slot.change(&neighbor_change.new_protos);
                    changes.append(&mut self.propagate(&neighbor_change.clone(), map));
                }
            }
        }

        changes
    }

    pub fn propagate_all(&self, map: &mut Map) -> Vec<SlotChange> {
        self.change_each_slot(map, |position, map| {
            if let Some(slot) = map.get_slot(position) {
                return self.propagate(
                    &SlotChange {
                        position,
                        new_protos: slot.possibilities.clone(),
                    },
                    map,
                );
            }

            vec![]
        })

        // let mut changes = vec![];

        // let start = self.position;
        // let end = self.position + self.size;
        // for x in start.x..end.x {
        //     for y in start.y..end.y {
        //         for z in start.z..end.z {
        //             let position = Vector3i { x, y, z };
        //             let slot = map.get_slot(position);
        //             if let Some(slot) = slot {
        //                 changes.append(&mut self.propagate(
        //                     &SlotChange {
        //                         position: position,
        //                         new_protos: slot.possibilities.clone(),
        //                     },
        //                     map,
        //                 ))
        //             }
        //         }
        //     }
        // }

        // changes
    }

    fn change_each_slot<F: Fn(Vector3i, &mut Map) -> Vec<SlotChange>>(
        &self,
        map: &mut Map,
        f: F,
    ) -> Vec<SlotChange> {
        let mut changes = vec![];

        let start = self.position;
        let end = self.position + self.size;
        for x in start.x..end.x {
            for y in start.y..end.y {
                for z in start.z..end.z {
                    changes.append(&mut f(Vector3i { x, y, z }, map))
                }
            }
        }

        changes
    }

    fn map_filter_slots<F: Fn(Vector3i) -> Option<Vector3i>>(&self, f: F) -> Vec<Vector3i> {
        let mut slots = vec![];

        let start = self.position;
        let end = self.position + self.size;
        for x in start.x..end.x {
            for y in start.y..end.y {
                for z in start.z..end.z {
                    if let Some(position) = f(Vector3i { x, y, z }) {
                        slots.push(position);
                    }
                }
            }
        }

        slots
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
