use std::sync::mpsc::TryRecvError;

use godot::prelude::*;

use crate::models::{
    driver_update::ManagerUpdate,
    manager::{ManagerCommand, ManagerCommandType, ManagerState},
    phone::Phone,
};

use super::worker_pool::WorkerPool;

const NUM_THREADS: usize = 4;

pub struct Manager {
    state: ManagerState,

    // Receive commands from and send updates to the main thread
    phone: Phone<ManagerUpdate, ManagerCommand>,

    // Receive updates from and send commands to worker threads
    pool: WorkerPool,
}

impl Manager {
    pub fn new(
        phone: Phone<ManagerUpdate, ManagerCommand>,
        map_size: Vector3i,
        chunk_size: Vector3i,
        chunk_overlap: i32,
    ) -> Self {
        let pool = WorkerPool::new(NUM_THREADS, map_size, chunk_size, chunk_overlap);

        Self {
            state: ManagerState::Idle,
            phone,
            pool,
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
                        TryRecvError::Empty => match self.pool.manage_workers() {
                            Some(update) => self.report(update),
                            None => continue,
                        },
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
