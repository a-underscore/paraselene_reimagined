use super::CHUNK_SIZE;
use crate::util;
use hex::{anyhow, assets::Texture, nalgebra::Vector2, parking_lot::RwLock, Context};
use once_cell::sync::Lazy;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub const ASTEROID_1: &str = "asteroid_1";
pub const ASTEROID_2: &str = "asteroid_2";
pub const SPACE: &str = "space";
pub const METAL: &str = "metal";

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub id: String,
    pub position: [f32; 2],
    pub grid: Vec<Vec<Option<String>>>,
}

pub struct Tile {
    pub max: f64,
    pub min: f64,
    pub rand: f64,
    pub texture: Texture,
    pub id: String,
}

impl Tile {
    pub fn check(&self, rng: &mut StdRng, value: f64) -> Option<(&String, &Texture)> {
        if rng.gen_bool(self.rand) && self.max >= value && self.min <= value {
            Some((&self.id, &self.texture))
        } else {
            None
        }
    }

    pub fn asteroid_1(context: &Context) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            texture: util::load_texture(&context, &Self::file_map(ASTEROID_1).unwrap())?,
            id: ASTEROID_1.to_string(),
        }))
    }

    pub fn asteroid_2(context: &Context) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            max: 1.0,
            min: 0.25,
            rand: 1.0,
            texture: util::load_texture(&context, &Self::file_map(ASTEROID_2).unwrap())?,
            id: ASTEROID_2.to_string(),
        }))
    }

    pub fn metal(context: &Context) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            max: 1.0,
            min: 2.0 / 3.0,
            rand: 2.0 / 3.0,
            texture: util::load_texture(&context, &Self::file_map(METAL).unwrap())?,
            id: METAL.to_string(),
        }))
    }

    pub fn space(context: &Context) -> anyhow::Result<Arc<Texture>> {
        Ok(Arc::new(util::load_texture(
            &context,
            &Self::file_map(SPACE).unwrap(),
        )?))
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

#[derive(Clone)]
pub struct Chunk {
    pub position: Vector2<f32>,
    pub grid: Vec<Vec<Option<Arc<Tile>>>>,
}

impl Chunk {
    pub fn new(position: Vector2<f32>, grid: Vec<Vec<Option<Arc<Tile>>>>) -> anyhow::Result<Self> {
        Ok(Self { position, grid })
    }
}
