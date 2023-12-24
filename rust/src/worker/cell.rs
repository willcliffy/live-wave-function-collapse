use godot::prelude::*;
use rand::prelude::*;

use crate::models::{library::Book, prototype::Prototype};

#[derive(Clone, Debug)]
pub struct Cell {
    pub position: Vector3i,
    pub possibilities: Vec<Prototype>,

    // Book traits
    version: String,
    locked: bool,
}

impl Book for Cell {
    fn location(&self) -> Vector3i {
        self.position
    }

    fn version(&self) -> String {
        self.version.clone()
    }

    fn set_version(&mut self, version: String) {
        self.version = version;
    }

    fn is_checked_out(&self) -> bool {
        self.locked
    }

    fn check_out(&mut self) -> bool {
        if self.is_checked_out() {
            return false;
        }

        self.locked = true;
        true
    }

    fn check_in(&mut self) -> bool {
        if !self.is_checked_out() {
            return false;
        }

        self.locked = false;
        true
    }
}

impl Cell {
    pub fn new(position: Vector3i, possibilities: Vec<Prototype>) -> Self {
        Self {
            position,
            possibilities,
            version: "base".into(),
            locked: false,
        }
    }

    pub fn changes_from(&self, other: &Cell) -> Option<Cell> {
        let mut new_protos = vec![];
        let direction = other.position - self.position;

        for proto in self.possibilities.iter() {
            if proto.compatible_with_any(&other.possibilities, direction) {
                new_protos.push(proto.clone())
            }
        }

        if new_protos.len() != self.possibilities.len() {
            let mut changed = self.clone();
            changed.change(&new_protos);
            return Some(changed);
        }

        None
    }

    pub fn collapsed(&self, prototype: Option<Prototype>) -> Option<Cell> {
        let proto = match prototype {
            Some(proto) => proto,
            None => {
                let index = self.choose_weighted()?;
                let proto = self.possibilities.get(index)?;
                proto.clone()
            }
        };

        let mut collapsed = self.clone();
        collapsed.change(&vec![proto]);
        Some(collapsed)
    }

    pub fn entropy(&self) -> usize {
        self.possibilities.len()
    }

    fn _is_collapsed(&self) -> bool {
        self.possibilities.len() <= 1
    }

    // &mut self

    pub fn change(&mut self, prototypes: &Vec<Prototype>) -> bool {
        let old_length = self.possibilities.len();
        self.possibilities = prototypes.clone();
        self.possibilities.len() != old_length
    }

    fn choose_weighted(&self) -> Option<usize> {
        let sum_of_weights = self.possibilities.iter().fold(0.0, |l, p| l + p.weight);
        let mut selected_weight = rand::thread_rng().gen_range(0.0..sum_of_weights);
        for i in 0..self.possibilities.len() {
            selected_weight -= self.possibilities[i].weight;
            if selected_weight <= 0.0 {
                return Some(i);
            }
        }

        godot_error!(
            "selected a weight greater than sum_of_weights! sow: {}",
            sum_of_weights
        );
        None
    }
}
