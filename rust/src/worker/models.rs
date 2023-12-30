use std::sync::Arc;

use godot::prelude::*;

use crate::{map::cell::Cell, models::library::Library3D};

///

#[repr(i32)]
#[derive(Property, Debug, PartialEq)]
pub enum WorkerCommandType {
    NoOp = 0,
    Collapse = 1,
    Stop = 2,
}

pub struct WorkerCommand {
    pub command: WorkerCommandType,
    pub map: Arc<Library3D<Cell>>,
}

impl WorkerCommand {
    pub fn new(command: WorkerCommandType, map: Arc<Library3D<Cell>>) -> Self {
        Self { command, map }
    }
}

//

#[derive(Debug)]
pub enum WorkerUpdateStatus {
    Ok(Vec<Cell>),
    Done,
    Error(anyhow::Error),
}

#[derive(Debug)]
pub struct WorkerUpdate {
    pub chunk_index: usize,
    pub status: WorkerUpdateStatus,
}

impl WorkerUpdate {
    pub fn new(chunk_index: usize, status: WorkerUpdateStatus) -> Self {
        Self {
            chunk_index,
            status,
        }
    }
}
