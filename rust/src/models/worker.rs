use godot::prelude::*;

use crate::worker::cell::Cell;

///

#[repr(i32)]
#[derive(Property, Debug)]
pub enum WorkerCommandType {
    NOOP = 0,
    COLLAPSE = 1,
}

pub struct WorkerCommand {
    pub command: WorkerCommandType,
    pub cells: Vec<Cell>,
}

impl WorkerCommand {
    pub fn new(command: WorkerCommandType, cells: Vec<Cell>) -> Self {
        Self { command, cells }
    }
}

//

#[derive(Debug)]
pub struct WorkerUpdate {
    pub done: bool,
    pub changes: Vec<Cell>,
}

impl WorkerUpdate {
    pub fn new(done: bool, changes: Vec<Cell>) -> Self {
        Self { done, changes }
    }
}
