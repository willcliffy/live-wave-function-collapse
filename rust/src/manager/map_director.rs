use std::{
    cmp::{max, min},
    sync::Arc,
};

use godot::{builtin::Vector3i, log::godot_print};

use crate::{
    map::{
        cell::Cell,
        chunk::Chunk,
        models::{ChunkState, MapParameters},
    },
    models::{library::Library3D, prototype::Prototype},
};

use super::models::GetNextChunkResponse;

pub struct MapDirector {
    pub proto_data: Vec<Prototype>,
    pub library: Arc<Library3D<Cell>>,
    chunks: Vec<Chunk>,
}

impl MapDirector {
    pub fn new(params: &MapParameters) -> Self {
        let proto_data = Prototype::load();
        let library = params.generate_cell_library(&proto_data);
        let chunks = params.generate_chunks();
        Self {
            proto_data,
            library,
            chunks,
        }
    }

    pub fn get_next_chunk(&mut self) -> GetNextChunkResponse {
        let eligible_chunks: Vec<usize> = self
            .chunks
            .iter()
            .enumerate()
            .filter(|&(_, c)| c.state == ChunkState::Ready)
            .map(|(i, _)| i)
            .collect();

        if eligible_chunks.len() == 0 {
            return GetNextChunkResponse::NoChunksLeft;
        }

        let mut closest_to_edge_index = usize::MAX;
        let mut closest_to_edge_distance = i32::MAX;

        for i in eligible_chunks {
            // If any chunks below i are not collapsed, skip i
            if self.chunks.iter().any(|c| {
                c.state != ChunkState::Collapsed && c.position.y < self.chunks[i].position.y
            }) {
                continue;
            }

            let (start, end) = self.chunks[i].bounds();
            let overlapping = self
                .chunks
                .iter()
                .filter(|c| c.is_overlapping(&self.chunks[i]));

            // If any chunks that overlap with i are currently active, skip i
            if overlapping.clone().any(|c| c.state == ChunkState::Active) {
                continue;
            }

            if start.x > 0
                && start.z > 0
                && end.x < self.library.size.x
                && end.z < self.library.size.z
            {
                // If there are no collapsed chunks overlapping with i, skip i
                // We skip this constraint if the chunk is on the x or z edge of the map space.
                if overlapping
                    .clone()
                    .all(|c| c.state != ChunkState::Collapsed)
                {
                    continue;
                }
            }

            let chunk_position = self.chunks[i].position;

            let (start, end) = self.chunks[i].bounds();
            let distances = vec![
                start.x,
                start.z,
                self.library.size.x - min(end.x, self.library.size.x),
                self.library.size.z - min(end.z, self.library.size.z),
            ];

            let mut distance_to_edge = *distances.iter().min().unwrap();
            distance_to_edge += 1000 * chunk_position.y;

            if distance_to_edge < closest_to_edge_distance {
                closest_to_edge_distance = distance_to_edge;
                closest_to_edge_index = i;
            }
        }

        if closest_to_edge_index != usize::MAX {
            let changes;
            match self.reset_chunk(closest_to_edge_index) {
                Ok(changed) => changes = changed,
                Err(e) => return GetNextChunkResponse::Error(e),
            }
            self.set_chunk_state(closest_to_edge_index, ChunkState::Active);
            let chunk = self.chunks[closest_to_edge_index].clone();
            return GetNextChunkResponse::ChunkReady(closest_to_edge_index, chunk, changes);
        }

        return GetNextChunkResponse::NoChunksReady;
    }

    pub fn reset_chunk(&mut self, chunk_index: usize) -> anyhow::Result<Vec<Cell>> {
        let mut changes = vec![];

        // 1. Reset all overlap from other chunks
        match self.reset_overlap(chunk_index) {
            Ok(mut reset_changes) => {
                godot_print!(
                    "[M/WP {chunk_index}] Reset chunk. {} changes",
                    reset_changes.len()
                );
                changes.append(&mut reset_changes);
            }
            Err(e) => {
                godot_print!("[M/WP {chunk_index}] failed to apply reset chunk: {}", e);
                return Err(e);
            }
        }

        // 2. Apply custom constraints
        match self.apply_constraints(chunk_index) {
            Ok(mut constraint_changes) => {
                godot_print!(
                    "[M/WP {chunk_index}] Applied custom constraints. {} changes",
                    constraint_changes.len()
                );
                changes.append(&mut constraint_changes)
            }
            Err(e) => {
                godot_print!("[M/WP {chunk_index}] Failed to apply custom constraints: {e}",);
                return Err(e);
            }
        }

        // 3. Propagate in neighboring cells
        match self.propagate_neighbors(chunk_index) {
            Ok(mut propagate_changes) => {
                godot_print!(
                    "[M/WP {chunk_index}] Propagated from neighboring chunks. {} changes",
                    propagate_changes.len(),
                );
                changes.append(&mut propagate_changes);
            }
            Err(e) => {
                godot_print!(
                    "[M/WP {chunk_index}] Failed to propagate from neighboring chunks: {e}",
                );
                return Err(e);
            }
        }

        self.set_chunk_state(chunk_index, ChunkState::Ready);

        Ok(changes)
    }

    pub fn complete_chunk(&mut self, chunk_index: usize) {
        self.set_chunk_state(chunk_index, ChunkState::Collapsed);
    }

    fn set_chunk_state(&mut self, chunk_index: usize, state: ChunkState) -> Option<()> {
        let chunk = self.chunks.get_mut(chunk_index)?;
        chunk.state = state;
        Some(())
    }

    fn reset_overlap(&self, chunk_index: usize) -> anyhow::Result<Vec<Cell>> {
        let chunk;
        match self.chunks.get(chunk_index) {
            Some(c) => chunk = c,
            None => return Err(anyhow::anyhow!("Failed to retrieve chunk {}", chunk_index)),
        }

        let (start, end) = chunk.bounds();
        let mut range = self.library.check_out_range(start, end)?;
        let res = chunk.reset_cells(&mut range, &self.proto_data, self.library.size);
        self.library.check_in_range(&mut range)?;
        res
    }

    fn apply_constraints(&self, chunk_index: usize) -> anyhow::Result<Vec<Cell>> {
        let chunk;
        match self.chunks.get(chunk_index) {
            Some(c) => chunk = c,
            None => return Err(anyhow::anyhow!("Failed to retrieve chunk {}", chunk_index)),
        }

        let (start, end) = chunk.bounds();
        let mut range = self.library.check_out_range(start, end)?;
        let mut res = chunk.apply_constraints(&mut range, self.library.size);
        if res.is_ok() {
            res = chunk.propagate_all(&mut range);
        }
        self.library.check_in_range(&mut range)?;
        res
    }

    fn propagate_neighbors(&self, chunk_index: usize) -> anyhow::Result<Vec<Cell>> {
        let chunk;
        match self.chunks.get(chunk_index) {
            Some(c) => chunk = c,
            None => return Err(anyhow::anyhow!("Failed to retrieve chunk {}", chunk_index)),
        }
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
        let res = chunk.propagate_all(&mut range);
        self.library.check_in_range(&mut range)?;
        res
    }
}
