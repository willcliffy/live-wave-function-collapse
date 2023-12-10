use godot::prelude::*;

#[repr(i16)]
#[derive(Property)]
pub enum CollapserActionType {
    NOOP = 0,
    INIT = 1,
    START = 2,
    STOP = 3,
}

#[derive(GodotClass)]
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

#[repr(i16)]
#[derive(Property, PartialEq, Clone, Copy, Debug)]
pub enum CollapserState {
    NEW = 0,
    INITIALIZING = 1,
    INITIALIZED = 2,
    IDLE = 3,
    PROCESSING = 4,
    STOPPED = 5,
}

#[derive(GodotClass, Debug)]
pub struct CollapserUpdate {
    #[var]
    pub state: CollapserState,
    pub payload: Option<String>,
}

impl CollapserUpdate {
    pub fn new(state: CollapserState) -> Self {
        Self {
            state,
            payload: None,
        }
    }
}

///

#[derive(GodotClass)]
#[class(base=Node)]
struct MapParams {
    #[var]
    size: Vector3,

    #[var]
    chunk_size: Vector3,

    #[var]
    chunk_overlap: i32,

    #[base]
    node: Base<Node>,
}

#[godot_api]
impl INode for MapParams {
    fn init(node: Base<Node>) -> Self {
        Self {
            size: Vector3::ZERO,
            chunk_size: Vector3::ZERO,
            chunk_overlap: 0,
            node,
        }
    }
}
