use std::{
    cmp::{max, min},
    sync::Arc,
};

use godot::{builtin::Vector3i, log::godot_print};

use crate::models::{
    library::Library3D,
    manager::GetNextChunkResponse,
    map::{ChunkState, MapParameters},
    prototype::Prototype,
};

use super::{cell::Cell, chunk::Chunk};

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
            // If any chunks that overlap with i are currently active, skip i
            if self
                .chunks
                .iter()
                .filter(|c| c.is_overlapping(&self.chunks[i]))
                .any(|c| c.state == ChunkState::Active)
            {
                continue;
            }

            if self
                .chunks
                .iter()
                .any(|c| c.state != ChunkState::Active && c.position.y < self.chunks[i].position.y)
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

        let chunk = self.chunks.get_mut(chunk_index).unwrap();

        chunk.state = ChunkState::Initializing;

        // 1. Reset all overlap from other chunks
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

        // 2. Apply custom constraints
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

        // 3. Propagate in neighboring cells
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

        chunk.state = ChunkState::Ready;

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
}
