use crate::{game_manager::Player, Tag};
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
        event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
        keyboard::{KeyCode, PhysicalKey},
        window::Fullscreen,
    },
    world::{entity_manager::EntityManager, system_manager::System, World},
    Context, Control, Id,
};
use std::{cell::OnceCell, collections::HashMap, f32::consts::PI, sync::Arc};

#[derive(Eq, PartialEq, Hash)]
pub enum Input {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

pub const ZOOM: f32 = 5.0;

pub type Binds = HashMap<
    Input,
    Arc<
        dyn Fn(ElementState, Arc<RwLock<Context>>, Arc<RwLock<World>>) -> anyhow::Result<()>
            + Send
            + Sync,
    >,
>;

pub struct GameUiManager {
    player: Option<Id>,
    camera: Option<Id>,
    kp_cb: Binds,
}

impl GameUiManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            player: Default::default(),
            camera: Default::default(),
            kp_cb: Default::default(),
        })
    }

    pub fn add_keybind<F>(&mut self, i: Input, f: F)
    where
        F: Fn(ElementState, Arc<RwLock<Context>>, Arc<RwLock<World>>) -> anyhow::Result<()>
            + Send
            + Sync
            + 'static,
    {
        self.kp_cb.insert(i, Arc::new(f));
    }

    pub fn convert_state(state: ElementState) -> bool {
        match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        }
    }

    fn init_default_keybinds(&mut self, _: Arc<RwLock<World>>) {
        let player = self.player.unwrap();

        self.add_keybind(Input::Keyboard(KeyCode::KeyW), move |state, _, world| {
            if let Some(p) = world.read().em.read().get_component::<Player>(player) {
                p.write().states.forward = Self::convert_state(state);
            }

            Ok(())
        });
        self.add_keybind(Input::Keyboard(KeyCode::KeyS), move |state, _, world| {
            if let Some(p) = world.read().em.read().get_component::<Player>(player) {
                p.write().states.backward = Self::convert_state(state);
            }

            Ok(())
        });
        self.add_keybind(Input::Keyboard(KeyCode::KeyA), move |state, _, world| {
            if let Some(p) = world.read().em.read().get_component::<Player>(player) {
                p.write().states.left = Self::convert_state(state);
            }

            Ok(())
        });
        self.add_keybind(Input::Keyboard(KeyCode::KeyD), move |state, _, world| {
            if let Some(p) = world.read().em.read().get_component::<Player>(player) {
                p.write().states.right = Self::convert_state(state);
            }

            Ok(())
        });
    }
}

impl System for GameUiManager {
    fn init(&mut self, _: Arc<RwLock<Context>>, world: Arc<RwLock<World>>) -> anyhow::Result<()> {
        let em = world.read().em.clone();
        let em = em.read();

        self.player = Tag("player".to_string()).find(&em);

        self.init_default_keybinds(world);

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
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == context.read().window.id() => {
                control.write().exit = true;
            }
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(code),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } if window_id == context.read().window.id() => {
                if let Some(key) = self.kp_cb.get_mut(&Input::Keyboard(code)) {
                    key(state, context, world)?;
                }
            }

            Event::WindowEvent {
                window_id,
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } if window_id == context.read().window.id() => {
                if let Some(key) = self.kp_cb.get_mut(&Input::Mouse(button)) {
                    key(state, context, world)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
