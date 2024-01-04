use godot::prelude::*;

#[repr(i64)]
#[derive(Property, PartialEq, Clone, Copy, Debug)]
pub enum CollapserState {
    IDLE = 1,
    PROCESSING = 2,
    STOPPED = 3,
}
