use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use godot::prelude::*;

use crate::models::collapser_action::{CollapserAction, CollapserActionType};
use crate::models::driver_update::DriverUpdate;
use crate::worker::collapser::LWFCCollapser;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct LWFCDriver {
    // Thread I/O
    _handle: Option<JoinHandle<()>>,
    send_to_thread: Option<Sender<CollapserAction>>,
    recv_in_main: Option<Receiver<DriverUpdate>>,

    #[export]
    pub map_size: Vector3i,

    #[export]
    pub chunk_size: Vector3i,

    #[export]
    pub chunk_overlap: i32,

    #[base]
    node: Base<Node3D>,
}

#[godot_api]
impl INode3D for LWFCDriver {
    fn init(node: Base<Node3D>) -> Self {
        LWFCDriver {
            _handle: None,
            send_to_thread: None,
            recv_in_main: None,
            map_size: Vector3i { x: 15, y: 1, z: 15 },
            chunk_size: Vector3i { x: 9, y: 1, z: 9 },
            chunk_overlap: 2,
            node,
        }
    }

    fn ready(&mut self) {
        let (send_to_thread, recv_in_thread) = channel::<CollapserAction>();
        let (send_to_main, recv_in_main) = channel::<DriverUpdate>();

        self.send_to_thread = Some(send_to_thread);
        self.recv_in_main = Some(recv_in_main);

        let map_size = self.map_size.clone();
        let chunk_size = self.chunk_size.clone();
        let chunk_overlap = self.chunk_overlap.clone();

        let _handle = thread::spawn(move || {
            let mut collapser = LWFCCollapser::new(
                send_to_main,
                recv_in_thread,
                map_size,
                chunk_size,
                chunk_overlap,
            );
            collapser.run()
        });
    }

    fn process(&mut self, delta: f64) {
        for _ in 0..1 {
            self.tick(delta);
        }
    }

    fn exit_tree(&mut self) {
        self.stop()
    }
}

#[godot_api]
impl LWFCDriver {
    #[signal]
    fn map_initialized();

    #[signal]
    fn map_completed();

    #[signal]
    fn cells_changed(changes: Array<Dictionary>);

    #[func]
    pub fn start(&mut self) {
        self.send_action(CollapserActionType::START)
    }

    #[func]
    pub fn stop(&mut self) {
        self.send_action(CollapserActionType::STOP)
    }

    pub fn tick(&mut self, _delta: f64) -> Option<()> {
        let update = self.receive_update()?;
        if let Some(new_state) = update.new_state {
            godot_print!("Ignoring state update from thread: {:?}", new_state);
        }

        let changes = update.changes?;
        // godot_print!("Cells changed: {:?}", changes.len());
        let changes_array = Array::from_iter(changes.iter().map(|c| c.to_godot()));
        self.node
            .emit_signal("cells_changed".into(), &[changes_array.to_variant()]);

        Some(())
    }

    fn receive_update(&mut self) -> Option<DriverUpdate> {
        match &self.recv_in_main {
            Some(receiver) => match receiver.try_recv() {
                Ok(update) => Some(update),
                Err(_) => None,
            },
            None => {
                godot_error!("Tried to receive update, but there's no receiver!");
                None
            }
        }
    }

    fn send_action(&mut self, action_type: CollapserActionType) {
        match &self.send_to_thread {
            Some(sender) => match sender.send(CollapserAction::new(action_type)) {
                Ok(_) => (),
                Err(e) => godot_error!("Failed to send action! {}", e),
            },
            None => godot_error!("Tried to send action, but there's no sender!"),
        }
    }
}
