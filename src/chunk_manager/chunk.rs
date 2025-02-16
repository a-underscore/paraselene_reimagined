use crate::util;
use hex::{
    anyhow,
    assets::Shape,
    nalgebra::{Vector2, Vector4},
    parking_lot::RwLock,
    Context,
};
use hex_instance::components::Instance;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub const ASTEROID_1: &str = "asteroid_1";
pub const ASTEROID_2: &str = "asteroid_2";
pub const SPACE: &str = "space";
pub const METAL: &str = "metal";

#[derive(Clone)]
pub struct Chunk {
    pub position: Vector2<f32>,
    pub grid: Vec<Vec<Option<Arc<Tile>>>>,
}

impl Chunk {
    pub fn new(position: Vector2<f32>, grid: Vec<Vec<Option<Arc<Tile>>>>) -> anyhow::Result<Self> {
        Ok(Self { position, grid })
    }

    pub fn load(chunk_data: ChunkData, tiles: &HashMap<String, Arc<Tile>>) -> Self {
        Self {
            position: chunk_data.position.into(),
            grid: chunk_data
                .grid
                .into_iter()
                .map(|x| {
                    x.into_iter()
                        .map(|y| y.and_then(|id| tiles.get(&id).cloned()))
                        .collect()
                })
                .collect(),
        }
    }
}

pub struct ChunkType;

impl ChunkType {
    pub fn new() -> Arc<RwLock<ChunkType>> {
        Arc::new(RwLock::new(ChunkType))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub position: [f32; 2],
    pub grid: Vec<Vec<Option<String>>>,
}

pub struct Tile {
    pub max: f64,
    pub min: f64,
    pub rand: f64,
    pub instance: Arc<RwLock<Instance>>,
    pub id: String,
}

impl Tile {
    pub fn check(&self, rng: &mut StdRng, value: f64) -> Option<(&String, Arc<RwLock<Instance>>)> {
        if rng.gen_bool(self.rand) && self.max >= value && self.min <= value {
            Some((&self.id, self.instance.clone()))
        } else {
            None
        }
    }

    pub fn asteroid_1(context: &Context) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            instance: Self::new_instance(context, ASTEROID_1)?,
            id: ASTEROID_1.to_string(),
        }))
    }

    pub fn asteroid_2(context: &Context) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            instance: Self::new_instance(context, ASTEROID_2)?,
            id: ASTEROID_2.to_string(),
        }))
    }

    pub fn metal(context: &Context) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            max: 1.0,
            min: 2.0 / 3.0,
            rand: 2.0 / 3.0,
            instance: Self::new_instance(context, METAL)?,
            id: METAL.to_string(),
        }))
    }

    pub fn space(context: &Context) -> anyhow::Result<Arc<RwLock<Instance>>> {
        Self::new_instance(context, SPACE)
    }

    pub fn new_instance(context: &Context, id: &str) -> anyhow::Result<Arc<RwLock<Instance>>> {
        Instance::new(
            context,
            Arc::new(Shape::rect(context, Vector2::new(1.0, 1.0))?),
            Arc::new(util::load_texture(context, &Self::file_map(id).unwrap())?),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
            1,
        )
    }

    pub fn file_map(id: &str) -> Option<String> {
        match id {
            ASTEROID_1 => Some("art/asteroid.png".into()),
            ASTEROID_2 => Some("art/asteroid2.png".into()),
            METAL => Some("art/metal.png".into()),
            SPACE => Some("art/space.png".into()),
            _ => None,
        }
    }
}
