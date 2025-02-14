use crate::chunk_manager::Chunk;
use hex::parking_lot::RwLock;
use noise::Perlin;
use rand::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct State {
    pub rng: StdRng,
    pub perlin: Perlin,
    pub seed: u32,
}

impl State {
    pub fn new(seed: u32) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            rng: StdRng::seed_from_u64(seed as u64),
            perlin: Perlin::new(seed),
            seed,
        }))
    }
}
