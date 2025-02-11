use hex::{
    assets::*,
    components::{camera::*, *},
    nalgebra::*,
    parking_lot::RwLock,
    renderers::*,
    vulkano::image::sampler::Sampler,
    winit::{event_loop::EventLoop, window::WindowBuilder},
    world::{
        entity_manager::{component_manager::*, *},
        renderer_manager::*,
        system_manager::{System, *},
    },
    Control, *,
};
use image::{ImageFormat, ImageReader};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

pub fn load_texture(context: &Context, path: &str) -> anyhow::Result<Texture> {
    let mut img = ImageReader::open(path)?;

    img.set_format(ImageFormat::Png);

    let img = img.decode().unwrap().to_rgba8();
    let dims = img.dimensions();
    let img = img.into_raw();
    let sampler = Sampler::new(context.device.clone(), Default::default()).unwrap();

    Ok(Texture::new(context, sampler, &img, dims.0, dims.1).unwrap())
}

pub fn mouse_pos_world(
    dims: Vector2<f32>,
    camera_scale: Vector2<f32>,
    window_dims: (u32, u32),
    mouse_pos: (f64, f64),
) -> Option<Vector2<f32>> {
    let (x, y) = mouse_pos;
    let (width, height) = window_dims;

    Some(Vector2::new(
        camera_scale.x * ((x / width as f64) as f32 * dims.x - dims.x / 2.0),
        -camera_scale.y * ((y / height as f64) as f32 * dims.y - dims.y / 2.0),
    ))
}
