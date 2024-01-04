use godot::prelude::*;

#[repr(i16)]
#[derive(Property, Debug)]
pub enum CollapserActionType {
    NOOP = 0,
    START = 1,
    PAUSE = 2,
    STOP = 3,
}

#[derive(GodotClass, Debug)]
pub struct CollapserAction {
    pub action_type: CollapserActionType,
    pub payload: Option<String>,
}

impl CollapserAction {
    pub fn new(action_type: CollapserActionType) -> Self {
        Self {
            action_type,
            payload: None,
        }
    }
}
