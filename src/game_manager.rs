use crate::{tag::Tag, util};
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
use std::sync::Arc;

#[derive(Default)]
pub struct GameManager {
    pub player: Option<Id>,
    pub camera: Option<Id>,
    pub last_fs: bool,
    pub mouse_position: Vector2<f32>,
    pub dims: (u32, u32),
}

impl System for GameManager {
    fn init(
        &mut self,
        context: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let em = world.read().em.clone();
        let mut em = em.write();
        let player = em.add(true);

        self.player = Some(player);

        em.add_component(player, Tag::new("player"));
        em.add_component(
            player,
            Trans::new(Vector2::new(0.0, 0.0), 0.0, Vector2::new(1.0, 1.0)),
        );

        let shape = Arc::new(Shape::rect(&*context.read(), Vector2::new(1.0, 1.0))?);
        let instance = Instance::new(
            &*context.read(),
            shape.clone(),
            Arc::new(util::load_texture(&*context.read(), "art/player.png")?),
            Vector4::new(1.0, 1.0, 0.0, 1.0),
            0,
        )?;

        em.add_component(player, instance);

        let camera = em.add(true);

        em.add_component(camera, Tag::new("camera"));
        em.add_component(camera, Camera::new(Vector2::new(10.0, 10.0), 1000));
        em.add_component(
            camera,
            Trans::new(Vector2::new(0.0, 0.0), 0.0, Vector2::new(1.0, 1.0)),
        );

        self.camera = Some(camera);

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
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == context.read().window.id() => {
                control.write().exit = true;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(PhysicalSize { width, height }, ..),
                window_id,
            } if window_id == context.read().window.id() => {
                self.dims = (width, height);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
            } if window_id == context.read().window.id() => {
                let pos = Vector2::new(position.x as f32, position.y as f32);

                self.mouse_position = pos;
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == context.read().window.id() => {
                let player = self.player.unwrap();
                let camera_id = self.camera.unwrap();
                let em = world.read().em.clone();
                let em = em.read();
                let player_transform = em.get_component::<Trans>(player).unwrap();
                let camera = em.get_component::<Camera>(camera_id).unwrap();
                let camera = camera.read();
                let camera_transform = em.get_component::<Trans>(camera_id).unwrap();
                let camera_transform = camera_transform.read();
                let pos = util::mouse_pos_world(
                    camera.dimensions(),
                    camera_transform.scale(),
                    self.dims,
                    (self.mouse_position.x as f64, self.mouse_position.y as f64),
                )
                .unwrap_or_default();
                let mut player_transform = player_transform.write();
                let cross = Vector2::new(0.0, 1.0).perp(&pos);
                let angle = Vector2::new(0.0, 1.0).angle(&pos);
                let angle = if cross < 0.0 { angle } else { -angle };

                player_transform.set_rotation(angle);
            }
            _ => {}
        }

        Ok(())
    }
}
