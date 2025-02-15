pub mod input;
pub mod main_menu;

pub use input::Input;
pub use main_menu::MainMenu;

use crate::{
    player::{
        player_manager::CAM_DIMS,
        state::{GAME_MODE, MENU_MODE},
        Player, State,
    },
    Tag,
};
use hex::{
    anyhow,
    components::{Camera, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, Context, EntityManager, Id,
    },
    glium::glutin::{
        dpi::PhysicalPosition,
        event::{
            ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
            WindowEvent,
        },
        event_loop::ControlFlow,
    },
    math::Vec2d,
};
use std::{cell::OnceCell, collections::HashMap, f32::consts::PI};

pub const ZOOM: f32 = 5.0;

pub type Binds = HashMap<
    Input,
    Box<
        dyn Fn(
            ElementState,
            &mut Context,
            (&mut EntityManager, &mut ComponentManager),
        ) -> anyhow::Result<()>,
    >,
>;

pub struct GameUiManager {
    player: OnceCell<Option<Id>>,
    prefab: OnceCell<Option<Id>>,
    camera: OnceCell<Option<Id>>,
    kp_cb: Binds,
    main_menu: MainMenu,
    window_x: f32,
    window_y: f32,
}

impl GameUiManager {
    pub fn new(
        context: &Context,
        (window_x, window_y): (i32, i32),
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        Ok(Self {
            player: Default::default(),
            prefab: Default::default(),
            camera: Default::default(),
            kp_cb: Default::default(),
            main_menu: MainMenu::new(&context.display, (em, cm))?,
            window_x: window_x as f32,
            window_y: window_y as f32,
        })
    }

    pub fn add_keybind<F>(&mut self, i: Input, f: F)
    where
        F: Fn(
                ElementState,
                &mut Context,
                (&mut EntityManager, &mut ComponentManager),
            ) -> anyhow::Result<()>
            + 'static,
    {
        self.kp_cb.insert(i, Box::new(f));
    }

    // This will be replaced with values loaded from a configuration file.
    fn init_default_keybinds(&mut self, (em, cm): (&mut EntityManager, &mut ComponentManager)) {
        if let (Some(player), Some(prefab)) = (
            *self
                .player
                .get_or_init(|| Tag::new("player").find((em, cm))),
            *self
                .prefab
                .get_or_init(|| Tag::new("prefab").find((em, cm))),
        ) {
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::W),
                move |state, _, (_em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player) {
                        p.states.forward = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::S),
                move |state, _, (_em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player) {
                        p.states.backward = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::A),
                move |state, _, (_em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player) {
                        p.states.left = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::D),
                move |state, _, (_em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player) {
                        p.states.right = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Mouse(MouseButton::Left),
                move |state, _, (_em, cm)| {
                    let firing = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    if let Some(player) = cm.get_mut::<Player>(player) {
                        player.states.firing = firing;
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Mouse(MouseButton::Right),
                move |state, _, (_em, cm)| {
                    let removing = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    if let Some(player) = cm.get_mut::<Player>(player) {
                        player.states.removing = removing;
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::Tab),
                move |state, _, (_em, cm)| {
                    if let ElementState::Pressed = state {
                        if let Some(player) = cm.get_mut::<Player>(player) {
                            player.states.mode = (player.states.mode + 1) % player.hotbar.len();
                        }
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::R),
                move |state, _, (_em, cm)| {
                    if let ElementState::Pressed = state {
                        if let Some(transform) = cm.get_mut::<Transform>(prefab) {
                            transform.set_rotation(transform.rotation() % (2.0 * PI) + (PI / 2.0));
                        }
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::Escape),
                move |state, _, (_em, cm)| {
                    if let ElementState::Pressed = state {
                        if let Some(state) = cm.get_mut::<State>(player) {
                            state.mode = MENU_MODE;
                        }
                    }

                    Ok(())
                },
            );
        }
    }
}

impl System for GameUiManager {
    fn init(
        &mut self,
        _: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        self.init_default_keybinds((em, cm));

        Ok(())
    }

    fn update(
        &mut self,
        ev: &mut Ev,
        context: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        match ev {
            Ev::Event(Control {
                event: Event::MainEventsCleared,
                flow: _,
            }) => {
                if let Some(player) = *self
                    .player
                    .get_or_init(|| Tag::new("player").find((em, cm)))
                {
                    self.main_menu.update(player, (em, cm));

                    if let Some(pressed) = cm
                        .get_mut::<Callback>(self.main_menu.button)
                        .map(|c| c.check())
                    {
                        if pressed {
                            if let Some(state) = cm.get_mut::<State>(player) {
                                state.mode = GAME_MODE;
                            }
                        }
                    }
                }
            }
            Ev::Event(Control {
                flow,
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::CloseRequested,
                    },
            }) if *window_id == context.display.gl_window().window().id() => {
                *flow = Some(ControlFlow::Exit);
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event:
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        virtual_keycode: Some(code),
                                        state,
                                        ..
                                    },
                                ..
                            },
                        ..
                    },
                flow: _,
            }) if *window_id == context.display.gl_window().window().id() => {
                if let Some(key) = self.kp_cb.get_mut(&Input::Keyboard(*code)) {
                    key(*state, context, (em, cm))?;
                }
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::MouseInput { button, state, .. },
                        ..
                    },
                flow: _,
            }) if *window_id == context.display.gl_window().window().id() => {
                if let Some(key) = self.kp_cb.get_mut(&Input::Mouse(*button)) {
                    key(*state, context, (em, cm))?;
                }
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::MouseWheel { delta, .. },
                        ..
                    },
                flow: _,
            }) if *window_id == context.display.gl_window().window().id() => {
                if let Some(camera) = *self
                    .camera
                    .get_or_init(|| Tag::new("camera").find((em, cm)))
                {
                    let zoom_amount = {
                        let (_, y) = match delta {
                            MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                                (*x as f32, *y as f32)
                            }
                        };

                        y * 5.0
                    };

                    if let Some(camera) = cm.get_mut::<Camera>(camera) {
                        let (dimensions, z) = {
                            let (dimensions, z) = camera.dimensions();
                            let dimensions = {
                                let dimensions = dimensions
                                    - (Vec2d::new(
                                        zoom_amount / self.window_x,
                                        zoom_amount / self.window_y,
                                    ) * 2.0);

                                Vec2d::new(
                                    dimensions.x().clamp(
                                        1.0 / self.window_x * 2.0,
                                        (ZOOM * CAM_DIMS) / self.window_x * 10.0,
                                    ),
                                    dimensions.y().clamp(
                                        1.0 / self.window_y * 2.0,
                                        (ZOOM * CAM_DIMS) / self.window_y * 10.0,
                                    ),
                                )
                            };

                            (dimensions, z)
                        };

                        camera.set_dimensions((dimensions, z));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
