//! play :)

use mela::asset::tilemap::{Tilemap, Orthogonal};
use std::cell::RefCell;
use mela::state::State;
use crate::world::MyWorld;
use mela::gfx::RenderContext;
use mela::game::IoState;
use std::time::Duration;
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::system::SystemCaller;
use crate::physics::{BallComponent, PhysicsAnimator};
use mela::ecs::component::Transform;
use std::rc::Rc;

pub struct Play {
    world: MyWorld,
    systems: Vec<Box<dyn SystemCaller<MyWorld>>>
}

impl Play {
    pub fn new() -> Play {
        let snapshots = Rc::new(RefCell::new(Vec::new()));

        Play {
            world: MyWorld::new()
                .register::<BallComponent>()
                .register::<Transform<f64>>(),

            systems: vec![
                Box::new(PhysicsAnimator::<f64>::new(Rc::clone(&snapshots))) as Box<dyn SystemCaller<MyWorld>>
            ]
        }
    }
}

impl DebugDrawable for Play {}

impl State for Play {
    type Wrapper = Self;

    fn name(&self) -> &str {
        "Play"
    }

    fn update(mut self, delta: Duration, io_state: &IoState, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) -> Self::Wrapper {
        for system in &mut self.systems {
            system.dispatch(&self.world, delta, io_state, render_ctx, debug_ctx);
        }

        self
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        ()
    }
}