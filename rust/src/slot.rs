use godot::{builtin::math::ApproxEq, prelude::*};
use rand::prelude::*;

use crate::models::{driver_update::SlotChange, prototype::Prototype};

pub struct Slot {
    pub position: Vector3,
    pub possibilities: Vec<Prototype>,
}

impl Slot {
    pub fn _expand(&mut self, prototypes: Vec<Prototype>) -> Option<SlotChange> {
        self.possibilities = prototypes;
        None // TODO
    }

    pub fn _constrain(&mut self, prototypes: Vec<Prototype>) -> Option<SlotChange> {
        self.possibilities = prototypes;
        None // TODO
    }

    pub fn _constrain_uncapped(&mut self, _direction: Vector3) -> Option<SlotChange> {
        // TODO - we don't use direction!
        self.possibilities
            .retain(|p| p.valid_neighbors[0].contains(&String::from("p-1")));
        None // TODO
    }

    pub fn collapse(&mut self, prototype: Option<Prototype>) -> Option<SlotChange> {
        if let Some(proto) = prototype {
            self.possibilities = vec![proto];
        } else {
            self.possibilities = vec![self.choose_weighted()];
        }
        None // TODO
    }

    pub fn _remove(&mut self, prototypes: Vec<Prototype>) -> Option<SlotChange> {
        for i in 0..prototypes.len() {
            if self.possibilities.contains(&prototypes[i]) {
                self.possibilities.remove(i);
            }
        }
        None // TODO
    }

    pub fn _entropy(self) -> usize {
        self.possibilities.len()
    }

    pub fn _is_collapsed(self) -> bool {
        self.possibilities.len() <= 1
    }

    pub fn _is_adjacent_to(self, other: Slot) -> bool {
        self.position.distance_to(other.position).approx_eq(&1.0)
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
