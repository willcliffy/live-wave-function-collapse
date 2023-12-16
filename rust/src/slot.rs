use godot::prelude::*;
use rand::prelude::*;

use crate::models::{driver_update::SlotChange, prototype::Prototype};

pub struct Slot {
    pub position: Vector3i,
    pub possibilities: Vec<Prototype>,
}

impl Slot {
    pub fn new(position: Vector3i, possibilities: Vec<Prototype>) -> Self {
        Self {
            position,
            possibilities,
        }
    }

    pub fn change(&mut self, prototypes: &Vec<Prototype>) -> Option<SlotChange> {
        let old_length = self.possibilities.len();

        self.possibilities = prototypes.clone();

        if self.possibilities.len() != old_length {
            return Some(SlotChange {
                position: self.position,
                new_protos: self.possibilities.clone(),
            });
        }

        None
    }

    pub fn collapse(&mut self, prototype: Option<Prototype>) -> Option<SlotChange> {
        let old_length = self.possibilities.len();

        if let Some(proto) = prototype {
            self.possibilities = vec![proto];
        } else {
            if let Some(selected) = self.choose_weighted() {
                self.possibilities = vec![selected.clone()];
            } else {
                godot_print!("overcollapsed! {}", self.position);
                self.possibilities = vec![];
            }
        }

        if self.possibilities.len() != old_length {
            return Some(SlotChange {
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

    fn choose_weighted(&mut self) -> Option<&Prototype> {
        let sum_of_weights = self.possibilities.iter().fold(0.0, |l, p| l + p.weight);
        let mut selected_weight = rand::thread_rng().gen_range(0.0..sum_of_weights);
        for prototype in self.possibilities.iter() {
            selected_weight -= prototype.weight;
            if selected_weight <= 0.0 {
                return Some(prototype);
            }
        }

        godot_error!(
            "selected a weight greater than sum_of_weights! sow: {}",
            sum_of_weights
        );

        self.possibilities.last()
    }
}
