pub mod chunk_manager;
pub mod game_manager;
pub mod game_ui_manager;
pub mod state;
pub mod tag;
pub mod util;

use chunk_manager::ChunkManager;
use game_manager::GameManager;
use hex::{
    nalgebra::*,
    threadpool::ThreadPool,
    vulkano::swapchain::PresentMode,
    winit::{event_loop::EventLoop, window::WindowBuilder},
    world::{entity_manager::*, renderer_manager::*, system_manager::*},
    *,
};
use hex_instance::renderers::InstanceRenderer;
use hex_physics::systems::PhysicsManager;
use rand::prelude::*;
use state::State;
use std::sync::Arc;
use tag::Tag;

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
        ThreadPool::new(num_cpus::get()),
        Vector4::new(0.5, 0.5, 0.5, 1.0),
    )
    .unwrap();

    let state = State::new(rand::thread_rng().gen::<u32>());
    let em = EntityManager::new();

    {
        let mut em = em.write();
        let s = em.add(true);

        em.add_component(s, Tag::new("state"));
        em.add_component(s, state.clone());
    }

    let mut sm = SystemManager::new();

    sm.add(1, PhysicsManager);
    sm.add(0, GameManager::default());
    sm.add(0, ChunkManager::new(&context.read(), state).unwrap());

    let mut rm = RendererManager::default();

    rm.add(InstanceRenderer);

    let world = World::new(em, sm, rm);

    Context::init(context, ev, world).unwrap();
}
