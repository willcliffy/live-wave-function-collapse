use godot::prelude::*;

use crate::models::{
    phone::Phone,
    worker::{WorkerCommand, WorkerCommandType, WorkerUpdate, WorkerUpdateStatus},
};

use super::chunk::Chunk;

pub struct Worker {
    phone: Phone<WorkerUpdate, WorkerCommand>,
    index: usize,
    chunk: Chunk,
}

impl Worker {
    pub fn new(phone: Phone<WorkerUpdate, WorkerCommand>, index: usize, chunk: Chunk) -> Self {
        Self {
            phone,
            index,
            chunk,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.tick() {
                Ok(stop) => {
                    if stop {
                        godot_print!("[W{}] Stopping normally", self.index);
                        break;
                    }
                }
                Err(e) => {
                    let update = WorkerUpdate::new(self.index, WorkerUpdateStatus::Error(e));
                    if let Err(e) = self.phone.send(update) {
                        godot_error!("[W] Failed to send error update: {}", e)
                    }
                    break;
                }
            }
        }
    }

    fn tick(&mut self) -> anyhow::Result<bool> {
        let command = &mut self.phone.wait()?;
        match command.command {
            WorkerCommandType::NOOP => Ok(false),
            WorkerCommandType::STOP => Ok(true),
            WorkerCommandType::COLLAPSE => {
                let (start, end) = self.chunk.bounds();
                let mut range = command.map.check_out_range(start, end)?;
                let changes = self.chunk.collapse_next(&mut range)?;
                command.map.check_in_range(&mut range)?;

                self.phone.send(WorkerUpdate::new(self.index, changes))?;
                Ok(false)
            }
        }
    }
}
