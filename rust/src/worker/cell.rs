use godot::prelude::*;
use rand::prelude::*;

use crate::models::{driver_update::CellChange, prototype::Prototype};

pub struct Cell {
    pub position: Vector3i,
    pub possibilities: Vec<Prototype>,
}

impl Cell {
    pub fn new(position: Vector3i, possibilities: Vec<Prototype>) -> Self {
        Self {
            position,
            possibilities,
        }
    }

    pub fn changes_from(&self, other: &CellChange) -> Option<CellChange> {
        let mut new_protos = vec![];
        let direction = other.position - self.position;

        for proto in self.possibilities.iter() {
            if proto.compatible_with_any(&other.new_protos, direction) {
                new_protos.push(proto.clone())
            }
        }

        if new_protos.len() != self.possibilities.len() {
            return Some(CellChange {
                position: self.position,
                new_protos,
            });
        }

        None
    }

    pub fn change(&mut self, prototypes: &Vec<Prototype>) -> Option<CellChange> {
        let old_length = self.possibilities.len();

        self.possibilities = prototypes.clone();

        if self.possibilities.len() != old_length {
            return Some(CellChange {
                position: self.position,
                new_protos: self.possibilities.clone(),
            });
        }

        None
    }

    pub fn collapse(&mut self, prototype: Option<Prototype>) -> Option<CellChange> {
        let old_length = self.possibilities.len();

        if let Some(proto) = prototype {
            self.possibilities = vec![proto];
        } else if let Some(selected) = self.choose_weighted() {
            self.possibilities = vec![selected];
        } else {
            godot_print!(
                "Tried to collapse but already overcollapsed! {}",
                self.position
            );
            self.possibilities = vec![];
        }

        if self.possibilities.len() != old_length {
            return Some(CellChange {
                position: self.position,
                new_protos: self.possibilities.clone(), // TODO - can we avoid cloning here?
            });
        }

        None
    }

    pub fn entropy(&self) -> usize {
        self.possibilities.len()
    }

    fn _is_collapsed(&self) -> bool {
        self.possibilities.len() <= 1
    }

    fn choose_weighted(&mut self) -> Option<Prototype> {
        let sum_of_weights = self.possibilities.iter().fold(0.0, |l, p| l + p.weight);
        let mut selected_weight = rand::thread_rng().gen_range(0.0..sum_of_weights);
        for prototype in self.possibilities.iter() {
            selected_weight -= prototype.weight;
            if selected_weight <= 0.0 {
                return Some(prototype.clone());
            }
        }

        godot_error!(
            "selected a weight greater than sum_of_weights! sow: {}",
            sum_of_weights
        );

        self.possibilities.last().cloned()
    }
}
