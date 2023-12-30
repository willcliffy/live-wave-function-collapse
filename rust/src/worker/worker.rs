use godot::log::godot_print;

use crate::{map::chunk::Chunk, models::phone::Phone};

use super::models::{WorkerCommand, WorkerCommandType, WorkerUpdate, WorkerUpdateStatus};

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
                        break;
                    }
                }
                Err(e) => {
                    let update = WorkerUpdate::new(self.index, WorkerUpdateStatus::Error(e));
                    let _ = self.phone.send(update);
                    break;
                }
            }
        }
    }

    fn tick(&mut self) -> anyhow::Result<bool> {
        let mut stop = false;
        let command = &mut self.phone.wait()?;
        match command.command {
            WorkerCommandType::NoOp => (),
            WorkerCommandType::Stop => stop = true,
            WorkerCommandType::Collapse => {
                let (start, end) = self.chunk.bounds();
                let mut range = command.map.check_out_range(start, end)?;
                let update = match self.chunk.collapse_next(&mut range) {
                    Ok(status) => WorkerUpdate::new(self.index, status),
                    Err(e) => WorkerUpdate::new(self.index, WorkerUpdateStatus::Error(e)),
                };

                command.map.check_in_range(&mut range)?;

                match &update.status {
                    WorkerUpdateStatus::Done => stop = true,
                    WorkerUpdateStatus::Error(_) => stop = true,
                    WorkerUpdateStatus::Ok(changes) => {
                        if changes.len() == 0 {
                            godot_print!(
                                "Worker {} reporting {} changes",
                                self.index,
                                changes.len()
                            )
                        }
                    }
                }

                self.phone.send(update)?;
            }
        }
        Ok(stop)
    }
}
