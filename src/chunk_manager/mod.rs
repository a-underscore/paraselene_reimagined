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
pub struct ChunkManager;

impl ChunkManager {
    /*
    pub fn gen_chunks(&self, pos: Vector2, state: &mut State) {
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
    */
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
            } if window_id == context.read().window.id() => {
            }
            _ => {}
        }

        Ok(())
    }
}
