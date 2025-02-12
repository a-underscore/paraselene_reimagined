pub mod chunk_data;

use crate::{state::State, tag::Tag, util};
use chunk_data::ChunkData;
use hex::{
    anyhow,
    assets::Shape,
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
use std::sync::Arc;

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
}

impl ChunkManager {
    pub fn new(state: Arc<RwLock<State>>) -> Self {
        Self { state }
    }
}

impl ChunkManager {
    pub fn gen_chunk(&self, pos: Vector2<f32>) -> anyhow::Result<ChunkData> {
        let state = self.state.read();
        let mut data = ChunkData::new(pos);

        for i in 0..data.grid.len() {
            for j in 0..data.grid[i].len() {
                let x = pos.x as f64 * CHUNK_SIZE as f64 + i as f64;
                let y = pos.y as f64 * CHUNK_SIZE as f64 + j as f64;
                let val = state.perlin.get([x / 25.0, y / 25.0, 0.0]);
                /*
                let tiles: Vec<_> = state
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
                    .unwrap_or((None, &state.space));

                data.grid[i][j] = id.as_ref().cloned();
                */
            }
        }

        Ok(data)
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
