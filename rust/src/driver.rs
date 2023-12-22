use std::thread::{self, JoinHandle};

use godot::prelude::*;

use crate::models::driver_update::ManagerUpdate;
use crate::models::manager::{ManagerCommand, ManagerCommandType};
use crate::models::phone::Phone;
use crate::worker::manager::Manager;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct LWFCDriver {
    _manager_handle: JoinHandle<()>,
    phone_to_manager: Phone<ManagerCommand, ManagerUpdate>,

    #[export]
    map_size: Vector3i,

    #[base]
    node: Base<Node3D>,
}

#[godot_api]
impl INode3D for LWFCDriver {
    fn init(node: Base<Node3D>) -> Self {
        let map_size = Vector3i {
            x: 30,
            y: 15,
            z: 30,
        };

        let (phone_to_manager, phone_to_main) = Phone::<ManagerCommand, ManagerUpdate>::new_pair();

        let _manager_handle = thread::spawn(move || {
            let mut manager = Manager::new(phone_to_main, map_size);
            manager.run()
        });

        LWFCDriver {
            _manager_handle,
            phone_to_manager,
            map_size,
            node,
        }
    }

    fn process(&mut self, delta: f64) {
        self.tick(delta);
    }

    fn exit_tree(&mut self) {
        self.stop();
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
        self.send_command(ManagerCommandType::START)
    }

    #[func]
    pub fn stop(&mut self) {
        self.send_command(ManagerCommandType::STOP)
    }

    pub fn tick(&mut self, _delta: f64) -> Option<()> {
        let update = self.check_for_update()?;

        if let Some(new_state) = update.new_state {
            godot_print!("[D] Ignoring manager state update: {:?}", new_state);
        }

        let changes = update.changes?;
        let changes_array = Array::from_iter(changes.iter().map(|c| c.to_godot()));
        self.node
            .emit_signal("cells_changed".into(), &[changes_array.to_variant()]);

        Some(())
    }

    fn check_for_update(&mut self) -> Option<ManagerUpdate> {
        match self.phone_to_manager.check() {
            Ok(update) => Some(update),
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => None,
                std::sync::mpsc::TryRecvError::Disconnected => {
                    godot_print!("[D] Failed to receive update from manager: {}", e);
                    None
                }
            },
        }
    }

    fn send_command(&mut self, command: ManagerCommandType) {
        match self.phone_to_manager.send(ManagerCommand::new(command)) {
            Ok(_) => (),
            Err(e) => godot_error!("[D] Failed to send command to manager: {}", e),
        }
    }
}
