use super::CHUNK_SIZE;
use hex::nalgebra::Vector2;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub position: [f32; 2],
    pub grid: Vec<Vec<Option<String>>>,
}

impl ChunkData {
    pub fn new(position: Vector2<f32>) -> Self {
        Self {
            position: position.into(),
            grid: vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        }
    }
}
