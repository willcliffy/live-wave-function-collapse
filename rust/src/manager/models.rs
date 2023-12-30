use godot::prelude::*;

use crate::{
    map::{cell::Cell, chunk::Chunk},
    models::cell_change::CellChangeGodot,
};

///

#[repr(i32)]
#[derive(Property, PartialEq, Clone, Copy, Debug)]
pub enum ManagerState {
    Idle = 1,
    Working = 2,
    Stopped = 3,
}

///

#[repr(i32)]
#[derive(Property, Debug)]
pub enum ManagerCommandType {
    NoOp = 0,
    Start = 1,
    Pause = 2,
    Stop = 3,
}

#[derive(GodotClass, Debug)]
pub struct ManagerCommand {
    pub command: ManagerCommandType,
    pub payload: Option<String>,
}

impl ManagerCommand {
    pub fn new(command: ManagerCommandType) -> Self {
        Self {
            command,
            payload: None,
        }
    }
}

//

#[derive(GodotClass, Debug)]
pub struct ManagerUpdate {
    pub new_state: Option<ManagerState>,
    pub changes: Option<Vec<CellChangeGodot>>,
}

impl ManagerUpdate {
    pub fn new(new_state: Option<ManagerState>, changes: Option<Vec<CellChangeGodot>>) -> Self {
        Self { new_state, changes }
    }

    pub fn new_state(new_state: ManagerState) -> Self {
        ManagerUpdate::new(Some(new_state), None)
    }

    pub fn new_changes(changes: Vec<Cell>) -> Self {
        ManagerUpdate::new(
            None,
            Some(
                changes
                    .iter()
                    .map(|c| CellChangeGodot::from_internal(c.position, c.possibilities.clone()))
                    .collect(),
            ),
        )
    }
}

//

pub enum GetNextChunkResponse {
    NoChunksLeft,
    NoChunksReady,
    ChunkReady(usize, Chunk, Vec<Cell>),
    Error(anyhow::Error),
}

#[derive(PartialEq)]
pub enum WorkerPoolState {
    Healthy,
    Deadlocked,
}
