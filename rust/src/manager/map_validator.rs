use std::collections::HashMap;

use godot::{builtin::Vector3i, log::godot_print};

use crate::{map::cell::Cell, models::prototype::Prototype};

use super::map_director::MapDirector;

pub struct MapValidator {}

pub enum PruneResult {
    Ok(Vec<Cell>),
    NoEffect,
    Error(anyhow::Error),
}

impl MapValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn prune_dead_cells(&self, map_director: &mut MapDirector) -> PruneResult {
        godot_print!("starting prune");
        let mut range;
        match map_director
            .library
            .check_out_range(Vector3i::ZERO, map_director.library.size)
        {
            Ok(r) => range = r,
            Err(e) => return PruneResult::Error(e),
        }

        let mut traversal_list: Vec<Vector3i> = vec![];
        let mut visited: HashMap<usize, ()> = HashMap::new();

        for x in range.start.x..range.end.x {
            let live_1 = Vector3i {
                x,
                y: 0,
                z: range.start.z,
            };
            let cell = range.books.get(range.index(live_1)).unwrap();
            match cell.entropy() {
                0 => godot_print!("overcollapsed along edge during prune: {live_1}"),
                1 => {
                    let proto = cell.possibilities.first().unwrap();
                    if proto.id != "p-1" {
                        visited.insert(range.index(live_1), ());
                        traversal_list.push(live_1);
                    }
                }
                _ => {}
            };

            let live_2 = Vector3i {
                x,
                y: 0,
                z: range.end.z - 1,
            };
            let cell = range.books.get(range.index(live_2)).unwrap();
            match cell.entropy() {
                0 => godot_print!("overcollapsed along edge during prune: {live_2}"),
                1 => {
                    let proto = cell.possibilities.first().unwrap();
                    if proto.id != "p-1" {
                        visited.insert(range.index(live_2), ());
                        traversal_list.push(live_2);
                    }
                }
                _ => {}
            };
        }

        for z in range.start.z..range.end.z {
            let live_1 = Vector3i {
                x: range.start.x,
                y: 0,
                z,
            };
            let cell = range.books.get(range.index(live_1)).unwrap();
            match cell.entropy() {
                0 => godot_print!("overcollapsed along edge during prune: {live_1}"),
                1 => {
                    let proto = cell.possibilities.first().unwrap();
                    if proto.id != "p-1" {
                        visited.insert(range.index(live_1), ());
                        traversal_list.push(live_1);
                    }
                }
                _ => {}
            };

            let live_2 = Vector3i {
                x: range.end.x - 1,
                y: 0,
                z,
            };
            let cell = range.books.get(range.index(live_2)).unwrap();
            match cell.entropy() {
                0 => godot_print!("overcollapsed along edge during prune: {live_2}"),
                1 => {
                    let proto = cell.possibilities.first().unwrap();
                    if proto.id != "p-1" {
                        visited.insert(range.index(live_2), ());
                        traversal_list.push(live_2);
                    }
                }
                _ => {}
            };
        }

        while let Some(position) = traversal_list.pop() {
            let cell = range.books.get(range.index(position)).unwrap();
            let proto = cell.possibilities.first().unwrap();

            if cell.entropy() > 1 {
                godot_print!(
                    "[WARN] ignoring uncollapsed cell in prune traversal list: {position} with entropy {}",
                    cell.entropy()
                );
                continue;
            }

            for neighbor_position in range.get_neighbors(position) {
                let neighbor_index = range.index(neighbor_position);
                if visited.contains_key(&neighbor_index) {
                    continue;
                }

                let neighbor_cell = range.books.get(neighbor_index).unwrap();
                if neighbor_cell.entropy() != 1 {
                    continue;
                }

                let slot = Prototype::get_slot(proto, neighbor_position - position);
                if slot == "-1f" || slot == "-1" {
                    // godot_print!("skipping neighbor at position {neighbor_position} since position {position} has proto {:?}", proto);
                    continue;
                }

                visited.insert(neighbor_index, ());

                let neighbor_proto = neighbor_cell.possibilities.first().unwrap();
                if neighbor_proto.id == "p-1" {
                    godot_print!(
                        "[WARN] got p-1 cell in traversal. Slot {slot}: \n\t{position}{:?}\n\t{neighbor_position} p-1",
                        proto,
                    );
                    continue;
                }

                traversal_list.push(neighbor_position);
            }
        }

        let unit_protos: Vec<Prototype> = map_director
            .proto_data
            .iter()
            .filter(|p| p.id == "p-1")
            .cloned()
            .collect();
        if unit_protos.len() != 1 {
            return PruneResult::Error(anyhow::anyhow!("unit proto not 1: {:?}", unit_protos));
        }
        let unit_proto = unit_protos.first().unwrap();

        let mut changes = vec![];

        for (i, cell) in range.books.iter_mut().enumerate() {
            if visited.contains_key(&i) {
                continue;
            }

            if cell.possibilities.len() == 0 {
                godot_print!("Pruning: got overcollapsed cell at {}", cell.position);
                continue;
            }

            if cell.possibilities.len() > 1 {
                continue;
            }

            let proto = cell.possibilities.first().unwrap();
            if proto.id != "p-1" {
                if changes.len() == 0 {
                    godot_print!(
                        "{:?} {}",
                        cell.possibilities
                            .iter()
                            .map(|p| p.id.clone())
                            .collect::<Vec<String>>(),
                        proto.id
                    )
                }
                cell.change(&vec![unit_proto.clone()]);
                changes.push(cell.clone())
            }
        }

        godot_print!("cleared {} dead cells in prune!", changes.len());

        if let Err(e) = map_director.library.check_in_range(&mut range) {
            // TODO - might not want to squash error here
            godot_print!("Got error checking in prune result: {}", e);
        }

        if changes.len() > 0 {
            return PruneResult::Ok(changes);
        }

        PruneResult::NoEffect
    }
}
