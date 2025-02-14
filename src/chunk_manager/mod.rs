pub mod chunk;

pub use chunk::{Chunk, ChunkData, Tile, ASTEROID_1, ASTEROID_2, METAL, SPACE};

use crate::{state::State, tag::Tag, util};
use hex::{
    anyhow,
    assets::{Shape, Texture},
    components::{Camera, Trans},
    nalgebra::{Vector2, Vector4},
    parking_lot::RwLock,
    vulkano::{
        command_buffer::{
            allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
            CommandBufferUsage, RenderPassBeginInfo,
        },
        descriptor_set::allocator::StandardDescriptorSetAllocator,
        device::{
            physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
            QueueCreateInfo, QueueFlags,
        },
        format::Format,
        image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
        instance::{self, InstanceCreateFlags, InstanceCreateInfo},
        memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
        pipeline::graphics::viewport::Viewport,
        render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
        swapchain::{
            acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
            SwapchainPresentInfo,
        },
        sync::{self, GpuFuture},
        Validated, VulkanError, VulkanLibrary,
    },
    winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        window::Fullscreen,
    },
    world::{entity_manager::EntityManager, system_manager::System, World},
    Context, Control, Id,
};
use hex_instance::components::Instance;
use noise::NoiseFn;
use rand::prelude::*;
use std::{collections::HashMap, sync::Arc};

pub const MAX_MAP_SIZE: u32 = 10000;
pub const TILE_SIZE: u32 = 32;
pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_DIST: f32 = 1.0;
pub const MAX_CHUNK: u32 = MAX_MAP_SIZE / CHUNK_SIZE;
pub const MIN_CHUNK: u32 = 2;
pub const UNLOAD_BIAS: u32 = 8;
pub const FRAME_LOAD_AMOUNT: usize = 1;

pub struct ChunkManager {
    pub state: Arc<RwLock<State>>,
    pub tiles: HashMap<String, Arc<Tile>>,
    pub space: Arc<Texture>,
}

impl ChunkManager {
    pub fn new(context: &Context, state: Arc<RwLock<State>>) -> anyhow::Result<Self> {
        Ok(Self {
            state,
            tiles: Self::chunk_texture_map(context)?,
            space: Tile::space(context)?,
        })
    }

    fn chunk_texture_map(context: &Context) -> anyhow::Result<HashMap<String, Arc<Tile>>> {
        let mut map = HashMap::new();

        map.insert(ASTEROID_1.into(), Tile::asteroid_1(context)?);
        map.insert(ASTEROID_2.into(), Tile::asteroid_2(context)?);
        map.insert(METAL.into(), Tile::metal(context)?);

        Ok(map)
    }
}

impl ChunkManager {
    pub fn gen_chunk(&self, pos: Vector2<f32>) -> anyhow::Result<Chunk> {
        let mut grid = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        let mut state = self.state.write();

        for i in 0..(CHUNK_SIZE as usize) {
            for j in 0..(CHUNK_SIZE as usize) {
                let x = pos.x as f64 * CHUNK_SIZE as f64 + i as f64;
                let y = pos.y as f64 * CHUNK_SIZE as f64 + j as f64;
                let val = state.perlin.get([x / 25.0, y / 25.0, 0.0]);
                let tiles: Vec<_> = self
                    .tiles
                    .values()
                    .filter_map(|t| {
                        t.check(&mut state.rng, val)
                            .map(|(id, t)| (Some(id.clone()), t))
                    })
                    .collect();
                let (id, _) = tiles
                    .choose(&mut state.rng)
                    .cloned()
                    .unwrap_or((None, &self.space));

                grid[i][j] = id.as_ref().and_then(|id| self.tiles.get(id)).cloned();
            }
        }

        Chunk::new(pos, grid)
    }
}

impl System for ChunkManager {
    fn update(
        &mut self,
        control: Arc<RwLock<Control>>,
        context: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let event = control.read().event.clone();

        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == context.read().window.id() => {}
            _ => {}
        }

        Ok(())
    }
}
