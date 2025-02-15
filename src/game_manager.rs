use crate::{tag::Tag, util};
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Trans},
    nalgebra::{Vector2, Vector4},
    parking_lot::RwLock,
    winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
    },
    world::{system_manager::System, World},
    Context, Control, Id,
};
use hex_instance::components::Instance;
use std::sync::Arc;

pub const PLAYER_MOVE_SPEED: f32 = 10.0;

#[derive(Default)]
pub struct ButtonStates {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Default)]
pub struct Player {
    pub states: ButtonStates,
    pub velocity: Vector2<f32>,
}

impl Player {
    pub fn force(&self) -> Vector2<f32> {
        let mut force = Vector2::default();

        if self.states.forward {
            force.y += PLAYER_MOVE_SPEED;
        }

        if self.states.backward {
            force.y -= PLAYER_MOVE_SPEED;
        }

        if self.states.left {
            force.x -= PLAYER_MOVE_SPEED;
        }

        if self.states.right {
            force.x += PLAYER_MOVE_SPEED;
        }

        if force.magnitude() > 0.0 {
            force = force.normalize() * PLAYER_MOVE_SPEED;
        }

        force
    }
}

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

        em.add_component(player, Arc::new(RwLock::new(Player::default())));
        em.add_component(player, Tag::new("player"));
        em.add_component(
            player,
            Trans::new(Vector2::new(0.0, 100.0), 0.0, Vector2::new(1.0, 1.0)),
        );

        let shape = Arc::new(Shape::rect(&context.read(), Vector2::new(1.0, 1.0))?);
        let instance = Instance::new(
            &context.read(),
            shape.clone(),
            Arc::new(util::load_texture(&context.read(), "art/player.png")?),
            Vector4::new(1.0, 1.0, 0.0, 1.0),
            0,
        )?;

        em.add_component(player, instance);

        let camera = em.add(true);

        em.add_component(camera, Tag::new("camera"));
        em.add_component(camera, Camera::new(Vector2::new(25.0, 25.0), 1000));
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
                let camera = em.get_component::<Camera>(camera_id).unwrap();
                let camera = camera.read();
                let camera_transform = em.get_component::<Trans>(camera_id).unwrap();
                let mut camera_transform = camera_transform.write();
                let pos = util::mouse_pos_world(
                    camera.dimensions(),
                    camera_transform.scale(),
                    self.dims,
                    (self.mouse_position.x as f64, self.mouse_position.y as f64),
                )
                .unwrap_or_default();
                let player_transform = em.get_component::<Trans>(player).unwrap();
                let player_transform = &mut *player_transform.write();
                let cross = Vector2::new(0.0, 1.0).perp(&pos);
                let angle = Vector2::new(0.0, 1.0).angle(&pos);
                let angle = if cross < 0.0 { angle } else { -angle };

                player_transform.set_rotation(angle);

                let player = em.get_component::<Player>(player).unwrap();
                let player = &mut *player.write();

                player.velocity += player.force();

                println!("{}", player_transform.position());

                player_transform.set_position(player_transform.position() + player.velocity);
                camera_transform.set_position(player_transform.position());
            }
            _ => {}
        }

        Ok(())
    }
}
