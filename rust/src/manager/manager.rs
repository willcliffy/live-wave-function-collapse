use std::sync::mpsc::TryRecvError;

use godot::prelude::*;

use crate::{
    manager::models::ManagerCommandType, map::models::MapParameters, models::phone::Phone,
};

use super::{
    map_director::MapDirector,
    map_validator::MapValidator,
    models::{ManagerCommand, ManagerState, ManagerUpdate},
    worker_pool::WorkerPool,
};

const NUM_THREADS: usize = 1;

pub struct Manager {
    state: ManagerState,

    // Receive commands from and send updates to the main thread
    phone: Phone<ManagerUpdate, ManagerCommand>,

    // Receive updates from and send commands to worker threads
    pool: WorkerPool,

    // Other reports
    map_director: MapDirector,
    map_validator: MapValidator,
}

impl Manager {
    pub fn new(phone: Phone<ManagerUpdate, ManagerCommand>, map_params: MapParameters) -> Self {
        let pool = WorkerPool::new(NUM_THREADS);
        let map_director = MapDirector::new(&map_params);
        let map_validator = MapValidator::new();

        Self {
            state: ManagerState::Idle,
            phone,
            pool,
            map_director,
            map_validator,
        }
    }

    pub fn run(&mut self) {
        godot_print!("[M] Starting run");
        loop {
            match self.state {
                // When stopped, break. TODO - tell worker threads that we can stop, wait on them.
                ManagerState::Stopped => {
                    godot_print!("[M] exiting normally");
                    break;
                }

                // When idle, block until a command is received
                ManagerState::Idle => match self.phone.wait() {
                    Ok(command) => self.on_command_received(command),
                    Err(_) => self.set_state(ManagerState::Stopped),
                },

                // When working, check if a command has been received, but do not block
                ManagerState::Working => match self.phone.check() {
                    Ok(command) => self.on_command_received(command),
                    Err(e) => match e {
                        TryRecvError::Empty => {
                            match self
                                .pool
                                .manage_workers(&mut self.map_director, &mut self.map_validator)
                            {
                                Some(update) => {
                                    let new_state = update.new_state;
                                    self.report(update);
                                    if new_state == Some(ManagerState::Stopped) {
                                        break;
                                    }
                                }
                                None => continue,
                            }
                        }
                        TryRecvError::Disconnected => self.set_state(ManagerState::Stopped),
                    },
                },
            }
        }
    }

    fn on_command_received(&mut self, command: ManagerCommand) {
        godot_print!("[M] Command received: {:?}", command);
        match command.command {
            ManagerCommandType::NoOp => godot_print!("[M] noop!"),
            ManagerCommandType::Start => self.set_state(ManagerState::Working),
            ManagerCommandType::Pause => self.set_state(ManagerState::Idle),
            ManagerCommandType::Stop => self.set_state(ManagerState::Stopped),
        }
    }

    // HELPERS

    fn set_state(&mut self, new_state: ManagerState) {
        godot_print!("[M] State updated: {:?}", new_state);
        self.state = new_state;
        self.report(ManagerUpdate::new_state(new_state));
    }

    fn report(&mut self, update: ManagerUpdate) {
        if let Err(e) = self.phone.send(update) {
            godot_print!("[M] Failed to post changes! {}", e)
        }
    }
}
