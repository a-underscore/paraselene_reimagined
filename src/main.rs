pub mod game_manager;
pub mod tag;
pub mod util;

use game_manager::GameManager;
use hex::{
    nalgebra::*,
    vulkano::swapchain::{FullScreenExclusive, PresentMode},
    winit::window::Fullscreen,
    winit::{event_loop::EventLoop, window::WindowBuilder},
    world::{entity_manager::*, renderer_manager::*, system_manager::*},
    *,
};
use hex_instance::renderers::InstanceRenderer;
use hex_physics::systems::PhysicsManager;
use std::sync::Arc;

fn main() {
    let ev = EventLoop::new().unwrap();
    let wb = Arc::new(
        WindowBuilder::new()
            .with_title("Paraselene Reimagined")
            .build(&ev)
            .unwrap(),
    );
    let context = Context::new(
        &ev,
        wb,
        PresentMode::Immediate,
        Vector4::new(0.0, 0.0, 0.0, 1.0),
    )
    .unwrap();

    let em = EntityManager::new();
    let mut sm = SystemManager::new(8);

    sm.add(0, PhysicsManager);
    sm.add(0, GameManager::default());

    let mut rm = RendererManager::default();

    rm.add(InstanceRenderer);

    let world = World::new(em, sm, rm);

    Context::init(context, ev, world).unwrap();
}
