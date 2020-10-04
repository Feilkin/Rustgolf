//! play :)

use mela::asset::tilemap::{Tilemap, Orthogonal};
use std::cell::RefCell;
use mela::state::State;
use crate::world::MyWorld;
use mela::gfx::RenderContext;
use mela::game::IoState;
use std::time::{Duration, Instant};
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::system::SystemCaller;
use crate::physics::{BallComponent, PhysicsAnimator, Snapshot, PhysicsBody, Ball};
use mela::ecs::component::Transform;
use std::rc::Rc;
use mela::gfx::primitives::{PrimitiveComponent, PrimitiveShape};
use mela::ecs::world::World;
use mela::lyon;
use mela::nphysics::ncollide2d::na::{Isometry2, Point2, Vector2};

pub struct Play {
    world: MyWorld,
    systems: Vec<Box<dyn SystemCaller<MyWorld>>>
}

impl Play {
    pub fn new() -> Play {
        let mut snapshots = Vec::new();

        let mut seed = Snapshot::new(Vec::new());

        let mut world = MyWorld::new()
            .register::<BallComponent>()
            .register::<Transform<f64>>()
            .register::<PrimitiveComponent>();

        for i in 0 .. 25 {
            let f = i as f64;
            let x = 300. + f % 5. * 60.;
            let y = 100. + (f / 5.).floor() * 60.;

            seed.balls.push(PhysicsBody {
                body: Ball {
                    radius: 21.335,
                },
                position: Point2::new(x, y),
                velocity: Vector2::new(-(f % 5. - 2.) * 50., 0.),
                acceleration: Vector2::new(0., 0.),
            });

            world = world.add_entity()
                .with_component(PrimitiveComponent {
                    color: [1., 1., 1., 1.],
                    shape: PrimitiveShape::Ball(21.335)
                })
                .with_component(Transform(Isometry2::translation(x, y)))
                .with_component(BallComponent {
                    index: i,
                    hidden: false
                })
                .build();
        }

        snapshots.push(seed);

        let mut seed_index = 0;

        println!("Beginning calculation");
        let start_time = Instant::now();

        while let Some(next) = snapshots[seed_index].next_snapshot() {
            snapshots.push(next);
            seed_index += 1;
        }

        println!("Total events: {}", snapshots.len());
        println!("First event: {:?}", snapshots.first().unwrap().end_time);
        println!("Last event: {:?}", snapshots.last().unwrap().start_time);
        println!("Calculations took {:?}", start_time.elapsed());

        let snapshots = Rc::new(RefCell::new(snapshots));

        Play {
            world,
            systems: vec![
                Box::new(PhysicsAnimator::<f64>::new(Rc::clone(&snapshots))) as Box<dyn SystemCaller<MyWorld>>,
                Box::new(mela::gfx::primitives::PrimitiveRenderer::new())
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

        let ui = &debug_ctx.ui;

        use mela::imgui::im_str;
        ui.text(im_str!("FPS: {:.2}", ui.io().framerate));

        self
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        for system in &self.systems {
            system.render(render_ctx);
        }
    }
}