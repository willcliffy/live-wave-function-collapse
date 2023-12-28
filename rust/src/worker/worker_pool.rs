use std::{
    cmp::{max, min},
    collections::HashMap,
    sync::{mpsc::TryRecvError, Arc},
    thread,
};

use godot::{builtin::Vector3i, engine::utilities::ceili, log::godot_print};

use crate::models::{
    driver_update::ManagerUpdate,
    library::Library3D,
    manager::GetNextChunkResponse,
    phone::Phone,
    prototype::Prototype,
    worker::{WorkerCommand, WorkerCommandType, WorkerUpdate, WorkerUpdateStatus},
};

use super::{cell::Cell, chunk::Chunk, worker::Worker};

pub struct WorkerPool {
    pool_size: usize,
    phones: HashMap<usize, Phone<WorkerCommand, WorkerUpdate>>,
    proto_data: Vec<Prototype>,
    chunks: Vec<Chunk>,
    library: Arc<Library3D<Cell>>,
}

impl WorkerPool {
    pub fn new(
        pool_size: usize,
        map_size: Vector3i,
        chunk_size: Vector3i,
        chunk_overlap: i32,
    ) -> Self {
        let phones = HashMap::new();

        let proto_data = Prototype::load();
        let cells = generate_cells(map_size, &proto_data);
        let library = Arc::new(Library3D::new(map_size, cells));

        let chunks = generate_chunks(map_size, chunk_size, chunk_overlap);

        godot_print!("last chunk position: {}", chunks.last().unwrap().position);

        Self {
            pool_size,
            phones,
            proto_data,
            chunks,
            library,
        }
    }

    pub fn manage_workers(&mut self) -> Option<ManagerUpdate> {
        let mut changes = vec![];

        if self.phones.len() < self.pool_size {
            match self.try_assign_new_worker() {
                Ok(mut changed) => changes.append(&mut changed),
                Err(e) => {
                    godot_print!("[WP] Failed to assign new worker: {}", e);
                }
            }
        }

        if let Some(mut changed) = self.check_for_updates() {
            changes.append(&mut changed)
        }

        if changes.len() > 0 {
            return Some(ManagerUpdate::new_changes(changes));
        }

        None
    }

    pub fn try_assign_new_worker(&mut self) -> anyhow::Result<Vec<Cell>> {
        match self.get_next_chunk() {
            GetNextChunkResponse::NoChunksLeft => {
                // Want to allow the currently running threads to finish, then when they are all done, quit
                Ok(vec![])
            }
            GetNextChunkResponse::NoChunksReady => {
                // there are chunks that need to be collapsed, but none are ready yet.
                Ok(vec![])
            }
            GetNextChunkResponse::ChunkReady(chunk_index) => {
                let changes = self.prepare_chunk(chunk_index)?;

                let chunk;
                match self.chunks.get(chunk_index) {
                    Some(valid_chunk) => chunk = valid_chunk.clone(),
                    None => {
                        return Err(anyhow::anyhow!(
                            "Failed to get ready chunk at index {}",
                            chunk_index
                        ))
                    }
                }

                let (mut phone_to_worker, phone_to_manager) = Phone::new_pair();
                let _ = phone_to_worker.send(WorkerCommand::new(
                    WorkerCommandType::Collapse,
                    self.library.clone(),
                ));
                self.phones.insert(chunk_index, phone_to_worker);

                let _worker_handle = thread::spawn(move || {
                    let mut worker = Worker::new(phone_to_manager, chunk_index, chunk);
                    worker.run()
                });

                // godot_print!("[M/WP] worker {} spawned", chunk_index);
                Ok(changes)
            }
        }
    }

    fn get_next_chunk(&self) -> GetNextChunkResponse {
        let eligible_chunks: Vec<usize> = self
            .chunks
            .iter()
            .enumerate()
            .filter(|&(_, c)| !c.collapsed && !c.active)
            .map(|(i, _)| i)
            .collect();

        if eligible_chunks.len() == 0 {
            return GetNextChunkResponse::NoChunksLeft;
        }

        let mut closest_to_edge_index = usize::MAX;
        let mut closest_to_edge_distance = i32::MAX;

        for i in eligible_chunks {
            // If any chunks that overlap with i are currently active, skip i
            if self
                .chunks
                .iter()
                .filter(|c| c.is_overlapping(&self.chunks[i]))
                .any(|c| c.active)
            {
                continue;
            }

            if self
                .chunks
                .iter()
                .any(|c| !c.collapsed && c.position.y < self.chunks[i].position.y)
            {
                continue;
            }

            let chunk_position = self.chunks[i].position;

            let (start, end) = self.chunks[i].bounds();
            let distances = vec![
                start.x,
                start.z,
                self.library.size.x - min(end.x, self.library.size.x),
                self.library.size.z - min(end.y, self.library.size.y),
            ];

            let mut distance_to_edge = *distances.iter().min().unwrap();
            distance_to_edge += 1000 * chunk_position.y;

            if distance_to_edge < closest_to_edge_distance {
                closest_to_edge_distance = distance_to_edge;
                closest_to_edge_index = i;
            }
        }

        if closest_to_edge_index != usize::MAX {
            return GetNextChunkResponse::ChunkReady(closest_to_edge_index);
        }

        return GetNextChunkResponse::NoChunksReady;
    }

    fn prepare_chunk(&mut self, chunk_index: usize) -> anyhow::Result<Vec<Cell>> {
        let mut changes = vec![];

        let chunk = self.chunks.get_mut(chunk_index).unwrap();

        // 1. Set chunk as active
        {
            chunk.active = true;
            godot_print!(
                "[M/WP {}] set chunk at {} as active",
                chunk_index,
                chunk.position
            );
        }

        // 2. Reset all overlap from other chunks
        {
            let (start, end) = chunk.bounds();
            let mut range = self.library.check_out_range(start, end)?;
            match chunk.reset_cells(&mut range, &self.proto_data, self.library.size) {
                Ok(mut chunk_changes) => {
                    changes.append(&mut chunk_changes);
                    godot_print!("[M/WP {}] reset chunk at {}", chunk_index, chunk.position)
                }
                Err(e) => godot_print!("[M/WP {}] failed to apply reset chunk: {}", chunk_index, e),
            }
            self.library.check_in_range(&mut range)?;
        }

        // 3. Apply custom constraints
        {
            let (start, end) = chunk.bounds();
            let mut range = self.library.check_out_range(start, end)?;
            match chunk.apply_custom_constraints(&mut range, self.library.size) {
                Ok(()) => {
                    godot_print!(
                        "[M/WP {}] applied custom constraints to {}",
                        chunk_index,
                        chunk.position
                    )
                }
                Err(e) => godot_print!(
                    "[M/WP {}] failed to apply custom constraints: {}",
                    chunk_index,
                    e
                ),
            }
            self.library.check_in_range(&mut range)?;
        }

        // 4. Propagate in neighboring cells
        {
            let (start, end) = chunk.bounds();
            let range_start = Vector3i {
                x: max(0, start.x - 1),
                y: max(0, start.y - 1),
                z: max(0, start.z - 1),
            };
            let range_end = Vector3i {
                x: min(end.x + 1, self.library.size.x),
                y: min(end.y + 1, self.library.size.y),
                z: min(end.z + 1, self.library.size.z),
            };
            let mut range = self.library.check_out_range(range_start, range_end)?;
            match chunk.propagate_all(&mut range) {
                Ok(mut chunk_changes) => {
                    godot_print!(
                        "[M/WP {}] propagated {:?} changes at {}",
                        chunk_index,
                        chunk_changes.len(),
                        chunk.position
                    );
                    changes.append(&mut chunk_changes);
                }
                Err(e) => godot_print!("Error propagating chunk: {}", e),
            }
            self.library.check_in_range(&mut range)?;
        }

        Ok(changes)
    }

    fn check_for_updates(&mut self) -> Option<Vec<Cell>> {
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
                        done_queue.push(*chunk_index);
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
                        self.library.clone(),
                    ));
                    changes.append(&mut changed);
                }
                WorkerUpdateStatus::Done => {
                    done_queue.push(update.chunk_index);
                }
                WorkerUpdateStatus::Error(e) => {
                    godot_print!("[M/WP {}] Error: {}, resetting", update.chunk_index, e);
                    reset_queue.push(update.chunk_index);
                    done_queue.push(update.chunk_index);
                }
            }
        }

        for chunk_index in reset_queue {
            match self.prepare_chunk(chunk_index) {
                Ok(mut changed) => changes.append(&mut changed),
                Err(e) => {
                    godot_print!("[M/WP] Failed to reset chunk! {}", e)
                }
            }
            let chunk = self.chunks.get_mut(chunk_index)?;
            chunk.active = false;
            chunk.collapsed = false;
        }

        for chunk_index in done_queue {
            let chunk = self.chunks.get_mut(chunk_index)?;
            chunk.active = false;
            chunk.collapsed = true;
            self.phones.remove(&chunk_index);
        }

        if changes.len() > 0 {
            return Some(changes);
        }

        None
    }
}

fn generate_cells(size: Vector3i, all_protos: &Vec<Prototype>) -> Vec<Cell> {
    let mut cells = vec![];
    for y in 0..size.y {
        for x in 0..size.x {
            for z in 0..size.z {
                let mut cell = Cell::new(Vector3i { x, y, z }, all_protos.clone());

                if cell.position.y == 0 {
                    Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::DOWN);
                } else {
                    Prototype::retain_not_constrained(&mut cell.possibilities, "BOT".into());
                }

                if cell.position.y == size.y - 1 {
                    Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::UP);
                }

                if cell.position.x == 0 {
                    Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::LEFT);
                }

                if cell.position.x == size.x - 1 {
                    Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::RIGHT);
                }

                if cell.position.z == 0 {
                    Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::FORWARD);
                }

                if cell.position.z == size.z - 1 {
                    Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::BACK);
                }

                cells.push(cell);
            }
        }
    }
    cells
}

fn generate_chunks(map_size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Vec<Chunk> {
    let num_x = ceili((map_size.x / (chunk_size.x - chunk_overlap)) as f64) as i32;
    let num_y = ceili((map_size.y / (chunk_size.y - chunk_overlap)) as f64) as i32;
    let num_z = ceili((map_size.z / (chunk_size.z - chunk_overlap)) as f64) as i32;
    let position_factor = chunk_size - Vector3i::ONE * chunk_overlap;

    let mut chunks = vec![];
    for y in 0..num_y {
        for x in 0..num_x {
            for z in 0..num_z {
                let position = position_factor * Vector3i { x, y, z };

                let end = position + chunk_size;
                let mut clamped_chunk_size = chunk_size;
                if end.x > map_size.x {
                    clamped_chunk_size.x = map_size.x - position.x;
                }
                if end.y > map_size.y {
                    clamped_chunk_size.y = map_size.y - position.y;
                }
                if end.z > map_size.z {
                    clamped_chunk_size.z = map_size.z - position.z;
                }

                let new_chunk = Chunk::new(position, clamped_chunk_size);
                chunks.push(new_chunk);
            }
        }
    }

    chunks
}
