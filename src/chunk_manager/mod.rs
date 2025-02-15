pub mod chunk;

pub use chunk::{Chunk, ChunkData, ChunkType, Tile, ASTEROID_1, ASTEROID_2, METAL, SPACE};

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
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const MAX_MAP_SIZE: u32 = 10000;
pub const TILE_SIZE: u32 = 32;
pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_DIST: f32 = 1.0;
pub const MAX_CHUNK: u32 = MAX_MAP_SIZE / CHUNK_SIZE;
pub const MIN_CHUNK: u32 = 2;
pub const UNLOAD_BIAS: u32 = 8;
pub const FRAME_LOAD_AMOUNT: usize = 1;
pub const SAVE_DIR: &str = "save";

pub struct ChunkManager {
    pub state: Arc<RwLock<State>>,
    pub tiles: HashMap<String, Arc<Tile>>,
    pub space: Arc<RwLock<Instance>>,
    pub camera: Option<Arc<RwLock<Camera>>>,
    pub player_transform: Option<Arc<RwLock<Trans>>>,
}

impl ChunkManager {
    pub fn new(context: &Context, state: Arc<RwLock<State>>) -> anyhow::Result<Self> {
        Ok(Self {
            state,
            tiles: Self::chunk_texture_map(context)?,
            space: Tile::space(context)?,
            camera: None,
            player_transform: None,
        })
    }

    fn chunk_texture_map(context: &Context) -> anyhow::Result<HashMap<String, Arc<Tile>>> {
        let mut map = HashMap::new();

        map.insert(ASTEROID_1.into(), Tile::asteroid_1(context)?);
        map.insert(ASTEROID_2.into(), Tile::asteroid_2(context)?);
        map.insert(METAL.into(), Tile::metal(context)?);

        Ok(map)
    }

    pub fn chunk_pos(pos: Vector2<f32>) -> (u32, u32) {
        let pos = pos / CHUNK_SIZE as f32;

        (pos.x.ceil() as u32, pos.y.ceil() as u32)
    }

    pub fn chunk_file((x, y): (u32, u32)) -> String {
        format!("{x},{y}.json")
    }

    pub fn gen_chunk(
        state: Arc<RwLock<State>>,
        space: Arc<RwLock<Instance>>,
        tiles: HashMap<String, Arc<Tile>>,
        pos: Vector2<f32>,
    ) -> anyhow::Result<ChunkData> {
        let mut grid = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        let mut state = state.write();

        for i in 0..(CHUNK_SIZE as usize) {
            for j in 0..(CHUNK_SIZE as usize) {
                let x = pos.x as f64 * CHUNK_SIZE as f64 + i as f64;
                let y = pos.y as f64 * CHUNK_SIZE as f64 + j as f64;
                let val = state.perlin.get([x / 25.0, y / 25.0, 0.0]);
                let t: Vec<_> = tiles
                    .values()
                    .filter_map(|t| {
                        t.check(&mut state.rng, val)
                            .map(|(id, t)| (Some(id.clone()), t.clone()))
                    })
                    .collect();
                let (id, _) = t
                    .choose(&mut state.rng)
                    .cloned()
                    .unwrap_or((None, space.clone()));

                grid[i][j] = id.as_ref().cloned();
            }
        }

        Ok(ChunkData {
            position: pos.into(),
            grid,
        })
    }

    pub fn load_chunk(
        world: Arc<RwLock<World>>,
        state: Arc<RwLock<State>>,
        space: Arc<RwLock<Instance>>,
        tiles: HashMap<String, Arc<Tile>>,
        chunk @ (x, y): (u32, u32),
    ) -> anyhow::Result<()> {
        let chunks_dir = PathBuf::from(SAVE_DIR).join("chunks");

        fs::create_dir_all(&chunks_dir)?;

        let em = world.read().em.clone();
        let mut em = em.write();
        let path = chunks_dir.join(Self::chunk_file(chunk));
        let data = if Path::exists(&path) {
            let content = fs::read_to_string(path)?;
            let data: ChunkData = serde_json::from_str(content.as_str())?;

            data
        } else {
            let data = Self::gen_chunk(
                state.clone(),
                space.clone(),
                tiles.clone(),
                Vector2::new(x as f32, y as f32),
            )?;
            let content = serde_json::to_string(&data)?;

            fs::write(path, content)?;

            data
        };

        let chunk = Chunk::load(data, tiles);

        for i in 0..(CHUNK_SIZE as usize) {
            for j in 0..(CHUNK_SIZE as usize) {
                let e = em.add(true);

                em.add_component(e, ChunkType::new());
                em.add_component(
                    e,
                    chunk.grid[i][j]
                        .as_ref()
                        .map(|c| c.instance.clone())
                        .unwrap_or(space.clone()),
                );
                em.add_component(
                    e,
                    Trans::new(
                        Vector2::new(
                            (CHUNK_SIZE * x as u32) as f32 + i as f32,
                            (CHUNK_SIZE * y as u32) as f32 + j as f32,
                        ),
                        0.0,
                        Vector2::new(1.0, 1.0),
                    ),
                );
            }
        }

        Ok(())
    }
}

impl System for ChunkManager {
    fn init(
        &mut self,
        _context: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let em = world.read().em.clone();
        let em = em.read();

        self.camera = em.get_component::<Camera>(Tag("camera".to_string()).find(&em).unwrap());
        self.player_transform =
            em.get_component::<Trans>(Tag("player".to_string()).find(&em).unwrap());

        Ok(())
    }

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
            _ => {
                let camera = self.camera.as_ref().unwrap().read();
                let player_transform = self.player_transform.as_ref().unwrap().read();
                let player_chunk = Self::chunk_pos(player_transform.position());
                let offset_x =
                    (camera.dimensions().x.ceil() / CHUNK_SIZE as f32 * CHUNK_DIST).ceil() as u32;
                let offset_y =
                    (camera.dimensions().y.ceil() / CHUNK_SIZE as f32 * CHUNK_DIST).ceil() as u32;
                let min = (
                    player_chunk
                        .0
                        .checked_sub(offset_x)
                        .unwrap_or_default()
                        .max(MIN_CHUNK),
                    player_chunk
                        .1
                        .checked_sub(offset_y)
                        .unwrap_or_default()
                        .max(MIN_CHUNK),
                );
                let max = (
                    (player_chunk.0 + offset_x).min(MAX_CHUNK),
                    (player_chunk.1 + offset_y).min(MAX_CHUNK),
                );

                for i in min.0..max.0 {
                    for j in min.1..max.1 {
                        let chunk = (i, j);
                        let pool = context.read().pool.clone();
                        let world = world.clone();
                        let state = self.state.clone();
                        let tiles = self.tiles.clone();
                        let space = self.space.clone();

                        pool.execute(move || {
                            Self::load_chunk(world, state, space, tiles, chunk).unwrap();
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
