use std::thread::{self, JoinHandle};

use godot::prelude::*;

use crate::manager::manager::Manager;
use crate::manager::models::{ManagerCommand, ManagerCommandType, ManagerState, ManagerUpdate};
use crate::map::models::MapParameters;
use crate::models::phone::Phone;

#[repr(i32)]
#[derive(Property, Clone, Export, PartialEq)]
pub enum DriverState {
    Stopped = 0,
    Running = 1,
}

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct LWFCDriver {
    _manager_handle: JoinHandle<()>,
    phone_to_manager: Phone<ManagerCommand, ManagerUpdate>,

    #[export]
    state: DriverState,

    #[export]
    map_size: Vector3i,

    #[base]
    node: Base<Node3D>,
}

#[godot_api]
impl INode3D for LWFCDriver {
    fn init(node: Base<Node3D>) -> Self {
        let map_size = Vector3i { x: 10, y: 1, z: 10 };
        let chunk_size = Vector3i { x: 12, y: 2, z: 12 };
        let chunk_overlap = 2;
        let map_parameters = MapParameters::new(map_size, chunk_size, chunk_overlap);

        let (phone_to_manager, phone_to_main) = Phone::<ManagerCommand, ManagerUpdate>::new_pair();

        let _manager_handle = thread::spawn(move || {
            let mut manager = Manager::new(phone_to_main, map_parameters);
            manager.run();
        });

        LWFCDriver {
            _manager_handle,
            phone_to_manager,
            state: DriverState::Running,
            map_size,
            node,
        }
    }

    fn process(&mut self, delta: f64) {
        if self.state == DriverState::Running {
            self.tick(delta);
        }
    }

    fn exit_tree(&mut self) {
        self.stop();
    }
}

#[godot_api]
impl LWFCDriver {
    #[signal]
    fn cells_changed(changes: Array<Dictionary>);

    #[func]
    pub fn start(&mut self) {
        self.send_command(ManagerCommandType::Start)
    }

    #[func]
    pub fn stop(&mut self) {
        self.send_command(ManagerCommandType::Stop)
    }

    pub fn tick(&mut self, _delta: f64) -> Option<()> {
        let update = self.check_for_update()?;

        if let Some(_new_state) = update.new_state {
            if _new_state == ManagerState::Stopped {
                self.state = DriverState::Stopped;
            }
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
            Err(e) => godot_print!("[D] Failed to send command to manager: {}", e),
        }
    }
}
