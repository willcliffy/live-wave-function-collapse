use godot::prelude::*;

///

#[repr(i32)]
#[derive(Property, PartialEq, Clone, Copy, Debug)]
pub enum ManagerState {
    IDLE = 1,
    PROCESSING = 2,
    STOPPED = 3,
}

///

#[repr(i32)]
#[derive(Property, Debug)]
pub enum ManagerCommandType {
    NOOP = 0,
    START = 1,
    PAUSE = 2,
    STOP = 3,
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
