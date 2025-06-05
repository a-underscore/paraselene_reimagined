use crate::{tag::Tag, util};
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Trans},
    nalgebra::{Matrix3, Vector2, Vector4},
    parking_lot::RwLock,
    winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
    },
    world::{system_manager::System, EntityManager, World},
    Context, Control, Id,
};
use hex_instance::components::Instance;
use hex_physics::components::Collider;
use std::{sync::Arc, time::Instant};

pub const PLAYER_ACCEL: f32 = 0.05;
pub const PLAYER_MAX_SPEED: f32 = 10.0;
pub const PLAYER_DECCEL_MUL: f32 = 0.1;

pub struct GameManager {
    pub player: Id,
    pub camera: Id,
    pub last_fs: bool,
    pub mouse_position: Vector2<f32>,
    pub dims: (u32, u32),
    pub last_frame: Instant,
}

impl GameManager {
    pub fn new(
        context: Arc<RwLock<Context>>,
        em: Arc<RwLock<EntityManager>>,
    ) -> anyhow::Result<Self> {
        let mut em = em.write();
        let player = em.add(true);

        em.add_component(player, Arc::new(RwLock::new(Player::default())));
        em.add_component(player, Tag::new("player"));
        em.add_component(
            player,
            Trans::new(Vector2::new(0.0, 100.0), 0.0, Vector2::new(1.0, 1.0)),
        );
        em.add_component(
            player,
            Collider::oct(
                Vector2::new(0.25, 0.25),
                [0].into(),
                [].into(),
                false,
                false,
            ),
        );

        let shape = Arc::new(Shape::rect(&context.read(), Vector2::new(1.0, 1.0))?);
        let instance = Instance::new(
            &context.read(),
            shape.clone(),
            Arc::new(util::load_texture(&context.read(), "art/player.png")?),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
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

        Ok(Self {
            player,
            camera,
            last_fs: Default::default(),
            mouse_position: Default::default(),
            dims: Default::default(),
            last_frame: Instant::now(),
        })
    }
}

impl System for GameManager {
    fn update(
        &mut self,
        control: Arc<RwLock<Control>>,
        context: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let event = control.read().event.clone();

        match event {
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
                let em = world.read().em.clone();
                let em = em.read();
                let camera = em.get_component::<Camera>(self.camera).unwrap();
                {
                    let mut camera = camera.write();
                    let (window_x, window_y) = {
                        let window_dims_x = self.dims.0 as i32;
                        let window_dims_y = self.dims.1 as i32;
                        let asp_ratio = self.dims.0 as f32 / self.dims.1 as f32;

                        (
                            window_dims_x as f32 / asp_ratio / 50.0,
                            window_dims_y as f32 / asp_ratio / 50.0,
                        )
                    };

                    camera.set_dimensions(Vector2::new(window_x, window_y));
                }
                let camera = camera.read();
                let camera_transform = em.get_component::<Trans>(self.camera).unwrap();
                let mut camera_transform = camera_transform.write();
                let pos = util::mouse_pos_world(
                    camera.dimensions(),
                    camera_transform.scale(),
                    self.dims,
                    (self.mouse_position.x as f64, self.mouse_position.y as f64),
                )
                .unwrap_or_default();
                let player_transform = em.get_component::<Trans>(self.player).unwrap();
                let player_transform = &mut *player_transform.write();
                let cross = Vector2::new(0.0, 1.0).perp(&pos);
                let angle = Vector2::new(0.0, 1.0).angle(&pos);
                let angle = if cross < 0.0 { angle } else { -angle };
                let now = Instant::now();
                let delta = now.duration_since(self.last_frame);

                self.last_frame = now;

                player_transform.set_rotation(angle);

                let player = em.get_component::<Player>(self.player).unwrap();
                let player = &mut *player.write();
                let f = player.force();
                let f = player.velocity
                    + if f.magnitude() != 0.0 {
                        (Matrix3::new_rotation(player_transform.rotation())
                            * util::lerp_vec2(f, Vector2::default(), 1.0).push(1.0))
                        .xy()
                            * PLAYER_ACCEL
                    } else {
                        -util::lerp_vec2(player.velocity, Vector2::default(), 1.0)
                            * PLAYER_ACCEL
                            * PLAYER_DECCEL_MUL
                    };
                player.velocity = if f.magnitude() != 0.0 {
                    f.normalize() * f.magnitude().min(PLAYER_MAX_SPEED)
                } else {
                    Vector2::default()
                };

                player_transform.set_position(
                    player_transform.position() + player.velocity * delta.as_secs_f32(),
                );
                camera_transform.set_position(player_transform.position());
            }
            _ => {}
        }

        Ok(())
    }
}

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
            force.y -= 1.0;
        }

        if self.states.backward {
            force.y += 1.0;
        }

        if self.states.left {
            force.x -= 1.0;
        }

        if self.states.right {
            force.x -= 1.0;
        }

        if force.magnitude() > 0.0 {
            force = force.normalize();
        }

        force
    }
}
