use std::{
    cmp::min,
    collections::HashMap,
    sync::{mpsc::TryRecvError, Arc},
    thread,
};

use godot::{
    builtin::Vector3i,
    log::{godot_error, godot_print},
};

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
        let cells = generate_cells(map_size);
        let library = Arc::new(Library3D::new(map_size, cells));

        let chunks = generate_chunks(map_size, chunk_size, chunk_overlap);

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

        while self.phones.len() < self.pool_size {
            match self.try_assign_new_worker() {
                Ok(mut changed) => {
                    if changed.len() > 0 {
                        changes.append(&mut changed);
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    godot_error!("Failed to assign new worker: {}", e);
                    break;
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
                    WorkerCommandType::COLLAPSE,
                    self.library.clone(),
                ));
                self.phones.insert(chunk_index, phone_to_worker);

                let _worker_handle = thread::spawn(move || {
                    let mut worker = Worker::new(phone_to_manager, chunk_index, chunk);
                    worker.run()
                });

                godot_print!("[M/WP] worker {} spawned", chunk_index);
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

            let (start, mut end) = self.chunks[i].bounds();

            end.x = min(end.x, self.library.size.x);
            end.y = min(end.y, self.library.size.y);
            end.z = min(end.z, self.library.size.z);

            let distances = vec![
                start.x,
                start.z,
                self.library.size.x - end.x,
                self.library.size.z - end.z,
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

        let mut neighboring_cells: Vec<Cell> = vec![];
        {
            let neighboring_chunks = self
                .chunks
                .iter()
                .filter(|c| c.is_overlapping(&self.chunks[chunk_index]))
                .collect::<Vec<&Chunk>>();

            let chunk;
            match self.chunks.get(chunk_index) {
                Some(valid_chunk) => chunk = valid_chunk,
                None => return Err(anyhow::anyhow!("No chunk at {}", chunk_index)),
            }

            for neighbor in neighboring_chunks {
                let (start, end) = neighbor.bounds();
                let range = self.library.copy_range(start, end)?;
                for cell_position in chunk.get_neighboring_cells(neighbor, self.library.size, 1) {
                    let i = range.index(cell_position, start);

                    // TODO - this breaks because get neighboring cells and index are not playing well together
                    // panic!();

                    match range.books.get(i) {
                        Some(cell) => {
                            let cell_clone = cell.clone();
                            neighboring_cells.push(cell_clone)
                        }
                        None => {
                            return Err(anyhow::anyhow!(
                                "Failed to get neighboring cell at {} (index {}). Range should be {} to {} but reported size {}",
                                cell_position,
                                i,
                                start,
                                end,
                                range.size
                            ));
                        }
                    }
                }
            }
        }

        {
            let chunk = self.chunks.get_mut(chunk_index).unwrap();
            chunk.active = true;
            godot_print!(
                "[M/WP {}] set chunk at {} as active",
                chunk_index,
                chunk.position
            );

            let (start, end) = chunk.bounds();
            let mut range = self.library.check_out_range(start, end)?;

            match chunk.reset_cells(&mut range, &self.proto_data, self.library.size) {
                Ok(mut chunk_changes) => {
                    changes.append(&mut chunk_changes);
                    godot_print!("[M/WP {}] reset chunk at {}", chunk_index, chunk.position)
                }
                Err(e) => godot_error!("[M/WP {}] failed to apply reset chunk: {}", chunk_index, e),
            }

            match chunk.propagate_all(&mut range) {
                Ok(mut chunk_changes) => changes.append(&mut chunk_changes),
                Err(e) => godot_error!("Error propagating chunk: {}", e),
            }

            match chunk.apply_custom_constraints(&mut range, self.library.size) {
                Ok(()) => {
                    godot_print!(
                        "[M/WP {}] applied custom constraints to {}",
                        chunk_index,
                        chunk.position
                    )
                }
                Err(e) => godot_error!(
                    "[M/WP {}] failed to apply custom constraints: {}",
                    chunk_index,
                    e
                ),
            }

            match chunk.propagate_all(&mut range) {
                Ok(mut chunk_changes) => changes.append(&mut chunk_changes),
                Err(e) => godot_error!("Error propagating chunk: {}", e),
            }

            match chunk.propagate_cells(&mut range, &neighboring_cells) {
                Ok(mut chunk_changes) => {
                    changes.append(&mut chunk_changes);
                }
                Err(e) => godot_error!(
                    "[M/WP {}] failed to apply propagate {} neighboring cells: {}",
                    chunk_index,
                    neighboring_cells.len(),
                    e
                ),
            }

            match chunk.propagate_all(&mut range) {
                Ok(mut chunk_changes) => changes.append(&mut chunk_changes),
                Err(e) => godot_error!("Error propagating chunk: {}", e),
            }

            self.library.check_in_range(&mut range)?;
        }

        Ok(changes)
    }

    fn check_for_updates(&mut self) -> Option<Vec<Cell>> {
        let mut updates = vec![];
        let mut completed = vec![];
        let mut resets = vec![];

        {
            for (chunk_index, phone) in self.phones.iter_mut() {
                match phone.check() {
                    Ok(update) => match update.status {
                        WorkerUpdateStatus::Ok(mut changes) => {
                            updates.append(&mut changes);

                            let _ = phone.send(WorkerCommand::new(
                                WorkerCommandType::COLLAPSE,
                                self.library.clone(),
                            ));
                        }
                        WorkerUpdateStatus::Done => {
                            // Set chunk completed, tell worker to stop
                            match self.chunks.get_mut(*chunk_index) {
                                Some(chunk) => {
                                    chunk.active = false;
                                    chunk.collapsed = true;
                                    completed.push(*chunk_index);
                                }
                                None => godot_error!(
                                    "[M/WP] failed to retrieve chunk at {}",
                                    chunk_index
                                ),
                            }
                        }
                        WorkerUpdateStatus::Reset(e) => {
                            godot_print!("[M/WP] reset error from worker {}: {}", chunk_index, e);
                            resets.push(*chunk_index);
                        }
                        WorkerUpdateStatus::Error(e) => {
                            // Report error
                            // Reset chunk
                            godot_print!("[M/WP] error from worker {}: {}", chunk_index, e);
                            match self.chunks.get_mut(*chunk_index) {
                                Some(chunk) => {
                                    chunk.active = false;
                                    chunk.collapsed = true;
                                    completed.push(*chunk_index);
                                }
                                None => godot_error!(
                                    "[M/WP] failed to retrieve chunk at {}",
                                    chunk_index
                                ),
                            }
                        }
                    },
                    Err(e) => match e {
                        TryRecvError::Empty => { /* No update from worker */ }
                        TryRecvError::Disconnected => {
                            godot_print!("[M/WP] disconnected from worker {}: {}", chunk_index, e);
                            match self.chunks.get_mut(*chunk_index) {
                                Some(chunk) => {
                                    chunk.active = false;
                                    chunk.collapsed = true;
                                    completed.push(*chunk_index);
                                }
                                None => godot_error!(
                                    "[M/WP] failed to retrieve chunk at {}",
                                    chunk_index
                                ),
                            }
                        }
                    },
                }
            }
        }

        for chunk_index in completed {
            self.phones.remove(&chunk_index);
            godot_print!("[M/WP {}] thread exited", chunk_index)
        }

        for chunk_index in resets {
            match self.prepare_chunk(chunk_index) {
                Ok(mut changed) => updates.append(&mut changed),
                Err(e) => {
                    godot_error!("[M/WP] Failed to reset chunk! {}", e)
                }
            }
        }

        if updates.len() > 0 {
            return Some(updates);
        }

        None
    }
}

fn generate_cells(size: Vector3i) -> Vec<Cell> {
    let mut cells = vec![];
    for y in 0..size.y {
        for x in 0..size.x {
            for z in 0..size.z {
                let cell = Cell::new(Vector3i { x, y, z }, vec![]);
                cells.push(cell);
            }
        }
    }
    cells
}

fn generate_chunks(size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Vec<Chunk> {
    let num_x =
        godot::engine::utilities::ceili((size.x / (chunk_size.x - chunk_overlap)) as f64) as i32;
    let num_y =
        godot::engine::utilities::ceili((size.y / (chunk_size.y - chunk_overlap)) as f64) as i32;
    let num_z =
        godot::engine::utilities::ceili((size.z / (chunk_size.z - chunk_overlap)) as f64) as i32;
    let position_factor = chunk_size - Vector3i::ONE * chunk_overlap;

    let mut chunks = vec![];
    for y in 0..num_y {
        for x in 0..num_x {
            for z in 0..num_z {
                let position = position_factor * Vector3i { x, y, z };
                let new_chunk = Chunk::new(position, chunk_size);
                chunks.push(new_chunk);
            }
        }
    }

    chunks
}
