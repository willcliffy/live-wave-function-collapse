use godot::prelude::*;

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

pub enum GetNextChunkResponse {
    NoChunksLeft,
    NoChunksReady,
    ChunkReady(usize),
}
