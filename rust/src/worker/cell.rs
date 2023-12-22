use godot::prelude::*;
use rand::prelude::*;

use crate::models::{driver_update::CellChange, prototype::Prototype};

use super::library::Book;

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
}

impl Cell {
    pub fn new(position: Vector3i, possibilities: Vec<Prototype>) -> Self {
        Self {
            position,
            possibilities,
            version: "".into(),
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
