use godot::prelude::*;

pub enum LWFCCollapserActionType {
    NOOP,
    INITIALIZE,
    START,
    STOP,
}

#[derive(GodotClass)]
pub struct LWFCCollapserAction {
    pub action_type: LWFCCollapserActionType,
    pub payload: Option<String>,
}
