use godot::prelude::*;
use rand::prelude::*;

use crate::models::{driver_update::SlotChange, prototype::Prototype};

pub struct Slot {
    pub position: Vector3,
    pub possibilities: Vec<Prototype>,
}

impl Slot {
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            possibilities: vec![],
        }
    }

    pub fn changed(&mut self, prototypes: Vec<Prototype>) -> Option<SlotChange> {
        self.possibilities = prototypes;

        unreachable!();
        None // TODO
    }

    pub fn _constrain_uncapped(&mut self, _direction: Vector3) -> Option<SlotChange> {
        // TODO - we don't use direction!
        let old_length = self.possibilities.len();
        self.possibilities
            .retain(|p| p.valid_neighbors[0].contains(&String::from("p-1")));

        if self.possibilities.len() != old_length {
            return Some(SlotChange {
                position: self.position,
                new_protos: self.possibilities.clone(), // TODO - can we avoid cloning here?
            });
        }

        None
    }

    pub fn collapse(&mut self, prototype: Option<Prototype>) -> Option<SlotChange> {
        if let Some(proto) = prototype {
            self.possibilities = vec![proto];
        } else {
            self.possibilities = vec![self.choose_weighted()];
        }

        unreachable!();
        None // TODO
    }

    pub fn _remove(&mut self, prototypes: Vec<Prototype>) -> Option<SlotChange> {
        for i in 0..prototypes.len() {
            if self.possibilities.contains(&prototypes[i]) {
                self.possibilities.remove(i);
            }
        }

        unreachable!();
        None // TODO
    }

    pub fn _entropy(self) -> usize {
        self.possibilities.len()
    }

    pub fn _is_collapsed(self) -> bool {
        self.possibilities.len() <= 1
    }

    fn choose_weighted(&mut self) -> Prototype {
        let sum_of_weights = self.possibilities.iter().fold(0, |l, p| l + p.weight);
        let mut selected_weight = rand::thread_rng().gen_range(0..sum_of_weights);
        for prototype in self.possibilities.iter() {
            selected_weight -= prototype.weight;
            if selected_weight <= 0 {
                return prototype.clone();
            }
        }

        return self.possibilities.last().unwrap().clone();
    }
}
