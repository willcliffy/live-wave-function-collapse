use std::collections::VecDeque;

use godot::engine::{INode3D, Node3D};
use godot::prelude::*;

use crate::collapser::LWFCCollapser;

#[derive(GodotClass)]
#[class(base=Node3D)]
struct LWFCDriver {
    proto_data: Option<Dictionary>,
    collapser: LWFCCollapser,

    collapsing: bool,

    #[base]
    node: Base<Node3D>,
}

#[godot_api]
impl INode3D for LWFCDriver {
    fn init(node: Base<Node3D>) -> Self {
        let proto_data = Option::None;
        let collapser = LWFCCollapser {
            queued_actions: VecDeque::new(),
            sender: None,
            idle: true,
        };
        let collapsing = false;
        Self {
            proto_data,
            collapser,
            collapsing,
            node,
        }
    }

    fn process(&mut self, _delta: f64) {
        if !self.collapsing {
            return;
        }
        if !self.collapser.idle {
            return;
        }
    }
}

#[godot_api]
impl LWFCDriver {
    // #[func]
    // fn increase_speed(&mut self, amount: f64) {
    //     self.speed += amount;
    //     self.sprite.emit_signal("speed_increased".into(), &[]);
    // }

    #[signal]
    fn map_initialized();

    #[signal]
    fn map_completed();

    #[signal]
    fn slot_created(slot: Vector3);

    #[signal]
    fn slot_constrained(slot: Vector3, protos: Array<GString>);

    #[signal]
    fn slot_reset(slot: Vector3, protos: Array<GString>);

    fn initialize_map() {}

    fn start_collapse(mut self) {
        self.collapsing = true;
    }

    fn stop_collapse(mut self) {
        self.collapsing = false;
    }
}
