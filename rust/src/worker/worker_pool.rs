use std::{collections::HashMap, sync::mpsc::TryRecvError, thread};

use godot::log::godot_print;

use crate::models::{
    driver_update::ManagerUpdate,
    manager::GetNextChunkResponse,
    phone::Phone,
    worker::{WorkerCommand, WorkerCommandType, WorkerUpdate, WorkerUpdateStatus},
};

use super::{cell::Cell, map_director::MapDirector, worker::Worker};

pub struct WorkerPool {
    pool_size: usize,
    phones: HashMap<usize, Phone<WorkerCommand, WorkerUpdate>>,
}

impl WorkerPool {
    pub fn new(pool_size: usize) -> Self {
        let phones = HashMap::new();

        Self { pool_size, phones }
    }

    pub fn manage_workers(&mut self, map_director: &mut MapDirector) -> Option<ManagerUpdate> {
        let mut changes = vec![];

        if self.phones.len() < self.pool_size {
            match self.try_assign_new_worker(map_director) {
                Ok(mut changed) => changes.append(&mut changed),
                Err(e) => {
                    godot_print!("[WP] Failed to assign new worker: {}", e);
                }
            }
        }

        if let Some(mut changed) = self.check_for_updates(map_director) {
            changes.append(&mut changed)
        }

        if changes.len() > 0 {
            return Some(ManagerUpdate::new_changes(changes));
        }

        None
    }

    pub fn try_assign_new_worker(
        &mut self,
        map_director: &mut MapDirector,
    ) -> anyhow::Result<Vec<Cell>> {
        match map_director.get_next_chunk() {
            GetNextChunkResponse::NoChunksLeft => {
                // Want to allow the currently running threads to finish, then when they are all done, quit
                Ok(vec![])
            }
            GetNextChunkResponse::NoChunksReady => {
                // there are chunks that need to be collapsed, but none are ready yet.
                Ok(vec![])
            }
            GetNextChunkResponse::ChunkReady(chunk_index, chunk, changes) => {
                let (mut phone_to_worker, phone_to_manager) = Phone::new_pair();
                let _ = phone_to_worker.send(WorkerCommand::new(
                    WorkerCommandType::Collapse,
                    map_director.library.clone(),
                ));
                self.phones.insert(chunk_index, phone_to_worker);

                let _worker_handle = thread::spawn(move || {
                    let mut worker = Worker::new(phone_to_manager, chunk_index, chunk);
                    worker.run()
                });

                // godot_print!("[M/WP] worker {} spawned", chunk_index);
                Ok(changes)
            }
            GetNextChunkResponse::Error(e) => Err(e),
        }
    }

    fn check_for_updates(&mut self, map_director: &mut MapDirector) -> Option<Vec<Cell>> {
        let mut update_queue: Vec<WorkerUpdate> = vec![];
        let mut reset_queue: Vec<usize> = vec![];
        let mut done_queue: Vec<usize> = vec![];

        for (chunk_index, phone) in self.phones.iter_mut() {
            match phone.check() {
                Ok(update) => update_queue.push(update),
                Err(e) => match e {
                    TryRecvError::Empty => { /* No update from worker */ }
                    TryRecvError::Disconnected => {
                        godot_print!("[M/WP {}] unexpected disconnect: {}", chunk_index, e);
                        reset_queue.push(*chunk_index);
                    }
                },
            }
        }

        let mut changes = vec![];
        for update in update_queue {
            match update.status {
                WorkerUpdateStatus::Ok(mut changed) => {
                    let phone = self.phones.get_mut(&update.chunk_index)?;
                    let _ = phone.send(WorkerCommand::new(
                        WorkerCommandType::Collapse,
                        map_director.library.clone(),
                    ));
                    changes.append(&mut changed);
                }
                WorkerUpdateStatus::Done => {
                    done_queue.push(update.chunk_index);
                }
                WorkerUpdateStatus::Error(e) => {
                    godot_print!("[M/WP {}] Error: {}, resetting", update.chunk_index, e);
                    reset_queue.push(update.chunk_index);
                }
            }
        }

        for chunk_index in reset_queue {
            match map_director.reset_chunk(chunk_index) {
                Ok(mut changed) => changes.append(&mut changed),
                Err(e) => {
                    godot_print!("[M/WP] Failed to reset chunk! {}", e)
                }
            }
            self.phones.remove(&chunk_index);
        }

        for chunk_index in done_queue {
            map_director.complete_chunk(chunk_index);
            self.phones.remove(&chunk_index);
        }

        if changes.len() > 0 {
            return Some(changes);
        }

        None
    }
}
