use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use godot::prelude::*;

use crate::models::{CollapserAction, CollapserActionType::*, CollapserState::*, CollapserUpdate};

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct LWFCCollapser {
    // Thread I/O
    handle: Option<JoinHandle<()>>,
    send_to_thread: Option<Sender<CollapserAction>>,
    receive_in_main: Option<Receiver<CollapserUpdate>>,

    #[var]
    pub idle: bool,
    #[var]
    pub collapsing: bool,

    #[base]
    node: Base<Node3D>,
}

#[godot_api]
impl INode3D for LWFCCollapser {
    fn init(node: Base<Node3D>) -> Self {
        LWFCCollapser {
            idle: true,
            collapsing: false,
            handle: None,
            send_to_thread: None,
            receive_in_main: None,
            node,
        }
    }
}

fn collapser_run(receiver: &mut Receiver<CollapserAction>, sender: &mut Sender<CollapserUpdate>) {
    godot_print!("Beginning thread");
    let mut state = NEW;
    loop {
        match state {
            INITIALIZING => {
                state = INITIALIZED;
                continue;
            }
            PROCESSING => {
                sender.send(CollapserUpdate::new(state)).unwrap();
                continue;
            }
            _default => (),
        }

        let action = receiver.try_recv().unwrap();
        match action.action_type {
            NOOP => continue,
            INIT => match state {
                NEW => state = INITIALIZED, // TODO
                INITIALIZING => godot_print!("LWFCCollapser INIT - Already initializing!"),
                _default => godot_print!("LWFCCollapser INIT - Already initialized!"),
            },
            START => match state {
                NEW => godot_print!("LWFCCollapser START - Not initialized!"),
                INITIALIZING => godot_print!("LWFCCollapser START - Still initializing!"),
                PROCESSING => godot_print!("LWFCCollapser START - Already running!"),
                _default => state = PROCESSING,
            },
            STOP => break,
        }
    }
}

#[godot_api]
impl LWFCCollapser {
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

    #[func]
    pub fn initialize(&mut self) {
        let (send_to_thread, mut recv_in_thread) = channel::<CollapserAction>();
        let (mut send_to_main, recv_in_main) = channel::<CollapserUpdate>();
        let handle = thread::spawn(move || collapser_run(&mut recv_in_thread, &mut send_to_main));

        send_to_thread.send(CollapserAction::new(INIT)).unwrap();

        self.send_to_thread = Some(send_to_thread);
        self.receive_in_main = Some(recv_in_main);
        self.handle = Some(handle);
        godot_print!("initialized!")
    }

    #[func]
    pub fn start(&mut self) {
        self.collapsing = true;
        let sender = self.send_to_thread.as_ref().unwrap();
        sender.send(CollapserAction::new(START)).unwrap();
    }

    #[func]
    pub fn tick(&mut self, _delta: f32) {
        let rcvr = self.receive_in_main.as_ref().unwrap();
        match rcvr.try_recv() {
            Ok(state_update) => match state_update.state {
                _default => godot_print!("{:?}", state_update),
            },
            Err(_) => (),
        }
    }

    #[func]
    pub fn stop(&mut self) {
        self.collapsing = true;
        let sender = self.send_to_thread.as_ref().unwrap();
        sender.send(CollapserAction::new(STOP)).unwrap();
    }
}
