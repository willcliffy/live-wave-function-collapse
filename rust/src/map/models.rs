use std::sync::Arc;

use godot::{builtin::Vector3i, engine::utilities::ceili};

use crate::models::{library::Library3D, prototype::Prototype};

use super::{cell::Cell, chunk::Chunk};

pub struct MapParameters {
    pub map_size: Vector3i,
    pub chunk_size: Vector3i,
    pub chunk_overlap: i32,
}

impl MapParameters {
    pub fn new(map_size: Vector3i, chunk_size: Vector3i, chunk_overlap: i32) -> Self {
        Self {
            map_size,
            chunk_size,
            chunk_overlap,
        }
    }

    pub fn generate_cell_library(&self, all_protos: &Vec<Prototype>) -> Arc<Library3D<Cell>> {
        let mut cells = vec![];
        for y in 0..self.map_size.y {
            for x in 0..self.map_size.x {
                for z in 0..self.map_size.z {
                    let mut cell = Cell::new(Vector3i { x, y, z }, all_protos.clone());

                    if cell.position.y == 0 {
                        Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::DOWN);
                    } else {
                        Prototype::retain_not_constrained(&mut cell.possibilities, "BOT".into());
                    }

                    if cell.position.y == self.map_size.y - 1 {
                        Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::UP);
                    }

                    if cell.position.x == 0 {
                        Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::LEFT);
                    }

                    if cell.position.x == self.map_size.x - 1 {
                        Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::RIGHT);
                    }

                    if cell.position.z == 0 {
                        Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::FORWARD);
                    }

                    if cell.position.z == self.map_size.z - 1 {
                        Prototype::retain_uncapped(&mut cell.possibilities, Vector3i::BACK);
                    }

                    cells.push(cell);
                }
            }
        }

        Arc::new(Library3D::new(self.map_size, cells))
    }

    pub fn generate_chunks(&self) -> Vec<Chunk> {
        let num_x = ceili(
            ((self.map_size.x + self.chunk_overlap) / (self.chunk_size.x - self.chunk_overlap))
                as f64,
        ) as i32;
        let num_y = ceili(
            ((self.map_size.y + self.chunk_overlap) / (self.chunk_size.y - self.chunk_overlap))
                as f64,
        ) as i32;
        let num_z = ceili(
            ((self.map_size.z + self.chunk_overlap) / (self.chunk_size.z - self.chunk_overlap))
                as f64,
        ) as i32;
        let position_factor = self.chunk_size - Vector3i::ONE * self.chunk_overlap;

        let mut chunks = vec![];
        for y in 0..num_y {
            for x in 0..num_x {
                for z in 0..num_z {
                    let position = position_factor * Vector3i { x, y, z };

                    let end = position + self.chunk_size;
                    let mut clamped_chunk_size = self.chunk_size;
                    if end.x > self.map_size.x {
                        clamped_chunk_size.x = self.map_size.x - position.x;
                    }
                    if end.y > self.map_size.y {
                        clamped_chunk_size.y = self.map_size.y - position.y;
                    }
                    if end.z > self.map_size.z {
                        clamped_chunk_size.z = self.map_size.z - position.z;
                    }

                    let new_chunk = Chunk::new(position, clamped_chunk_size);
                    chunks.push(new_chunk);
                }
            }
        }

        chunks
    }
}

//

#[derive(Clone, PartialEq, Debug)]
pub enum ChunkState {
    Ready,
    Active,
    Collapsed,
}
