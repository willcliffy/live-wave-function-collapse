use godot::prelude::*;

use crate::models::{
    phone::Phone,
    worker::{WorkerCommand, WorkerCommandType, WorkerUpdate},
};

use super::{cell::Cell, chunk::Chunk};

pub struct Worker {
    phone: Phone<WorkerUpdate, WorkerCommand>,
    chunk: Chunk,
}

impl Worker {
    pub fn new(phone: Phone<WorkerUpdate, WorkerCommand>, chunk: Chunk) -> Self {
        Self { phone, chunk }
    }

    pub fn run(&mut self) {
        loop {
            match self.phone.wait() {
                Ok(mut command) => match self.on_command_received(&mut command) {
                    Ok(result) => {
                        godot_print!("[W] command processed: {:?}", result);
                    }
                    Err(e) => {
                        godot_print!("[W] err processing command: {:?}", e);
                    }
                },
                Err(e) => {
                    godot_error!("[W] Disconnected: {}. Exiting.", e);
                    break;
                }
            };
        }
    }

    fn on_command_received(&mut self, command: &mut WorkerCommand) -> Result<Vec<Cell>, String> {
        godot_print!("[W] Message received: {:?}", command.command);
        match command.command {
            WorkerCommandType::NOOP => Ok(vec![]),
            WorkerCommandType::COLLAPSE => self.collapse_next(&mut command.cells),
        }
    }

    fn collapse_next(&mut self, cells: &mut Vec<Cell>) -> Result<Vec<Cell>, String> {
        self.chunk.collapse_next(cells);
        let update = WorkerUpdate::new(false, vec![]);
        self.post_changes(update);
        Ok(vec![])
    }

    // HELPERS

    fn post_changes(&mut self, update: WorkerUpdate) {
        if let Err(e) = self.phone.send(update) {
            godot_print!("[W] Failed to post changes! {}", e)
        }
    }
}
