use std::{collections::HashMap, thread};

use godot::log::godot_print;

use crate::{
    map::cell::Cell,
    models::phone::Phone,
    worker::{
        models::{WorkerCommand, WorkerCommandType, WorkerUpdate, WorkerUpdateStatus},
        worker::Worker,
    },
};

use super::{
    map_director::MapDirector,
    map_validator::MapValidator,
    models::{GetNextChunkResponse, ManagerState, ManagerUpdate, WorkerPoolState},
};

pub struct WorkerPool {
    state: WorkerPoolState,
    pool_size: usize,
    phones: HashMap<usize, Phone<WorkerCommand, WorkerUpdate>>,
}

impl WorkerPool {
    pub fn new(pool_size: usize) -> Self {
        let phones = HashMap::new();
        let state = WorkerPoolState::Healthy;

        Self {
            state,
            pool_size,
            phones,
        }
    }

    pub fn manage_workers(
        &mut self,
        map_director: &mut MapDirector,
        map_validator: &mut MapValidator,
    ) -> Option<ManagerUpdate> {
        let mut changes = vec![];

        if self.state == WorkerPoolState::Deadlocked {
            if self.phones.len() > 0 {
                return match self.check_for_updates(map_director) {
                    Some(changes) => Some(ManagerUpdate::new_changes(changes)),
                    None => None,
                };
            }

            return match map_validator.prune_dead_cells(map_director) {
                super::map_validator::PruneResult::Ok(changes) => {
                    godot_print!("[M/WP] Deadlock cleared, resuming normal execution");
                    self.state = WorkerPoolState::Healthy;
                    Some(ManagerUpdate::new_changes(changes))
                }
                super::map_validator::PruneResult::NoEffect => {
                    // Failure
                    godot_print!("[M/WP] Failed to clear deadlock, stopping manager");
                    Some(ManagerUpdate::new_state(ManagerState::Stopped))
                }
                super::map_validator::PruneResult::Error(e) => {
                    godot_print!(
                        "[M/WP] Error while clearing deadlock: {}. stopping manager",
                        e
                    );
                    Some(ManagerUpdate::new_state(ManagerState::Stopped))
                }
            };
        }

        if self.phones.len() < self.pool_size {
            match self.try_assign_new_worker(map_director) {
                Ok(mut changed) => changes.append(&mut changed),
                Err(e) => {
                    godot_print!("[WP] Failed to assign new worker: {}", e);
                    self.state = WorkerPoolState::Deadlocked;
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

                let (start, end) = chunk.bounds();
                godot_print!("[M/WP {chunk_index}] Thread spawning for chunk {start} - {end}");
                let _worker_handle = thread::spawn(move || {
                    let mut worker = Worker::new(phone_to_manager, chunk_index, chunk);
                    worker.run()
                });

                Ok(changes)
            }
            GetNextChunkResponse::Error(e) => Err(e),
        }
    }

    fn check_for_updates(&mut self, map_director: &mut MapDirector) -> Option<Vec<Cell>> {
        let mut update_queue: Vec<WorkerUpdate> = vec![];
        let mut reset_queue: Vec<usize> = vec![];
        let mut done_queue: Vec<usize> = vec![];

        for (_, phone) in self.phones.iter_mut() {
            loop {
                match phone.check() {
                    Ok(update) => update_queue.push(update),
                    Err(_) => break,
                }
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
            godot_print!("[M/WP {chunk_index}] Chunk completed");
            map_director.complete_chunk(chunk_index);
            self.phones.remove(&chunk_index);
        }

        if changes.len() > 0 {
            return Some(changes);
        }

        None
    }
}