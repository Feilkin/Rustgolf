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
use crate::physics::{BallComponent, PhysicsAnimator, Snapshot, PhysicsBody, Ball};
use mela::ecs::component::Transform;
use std::rc::Rc;
use mela::gfx::primitives::PrimitiveComponent;
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

        let mut seed = Snapshot {
            start_time: Duration::new(0, 0),
            end_time: Duration::new(u64::MAX, 999_999_999),
            balls: vec![
                PhysicsBody {
                    body: Ball {
                        radius: 21.335,
                    },
                    position: Point2::new(300., 100.),
                    velocity: Vector2::new(10., 0.),
                    acceleration: Vector2::new(10., 0.),
                },
                PhysicsBody {
                    body: Ball {
                        radius: 21.335,
                    },
                    position: Point2::new(600., 100.),
                    velocity: Vector2::new(-10., 0.),
                    acceleration: Vector2::new(-10., 0.),
                }
            ]
        };

        seed.next_snapshot();
        snapshots.push(seed.clone());

        while let Some(next) = seed.next_snapshot() {
            snapshots.push(next.clone());
            seed = next;
        }

        println!("First collision: {:.2}s", snapshots.first().unwrap().end_time.as_secs_f64());

        let snapshots = Rc::new(RefCell::new(snapshots));

        let circle = || {
            let mut builder = mela::lyon::path::Path::builder();
            builder.arc(lyon::math::Point::new(0.5, 0.5), lyon::math::Vector::new(0.5, 0.5), lyon::math::Angle::two_pi(), lyon::math::Angle::zero());
            builder.build()
        };

        Play {
            world: MyWorld::new()
                .register::<BallComponent>()
                .register::<Transform<f64>>()
                .register::<PrimitiveComponent>()
                .add_entity()
                .with_component(PrimitiveComponent {
                    color: [1., 1., 1., 1.],
                    shape: circle()
                })
                .with_component(Transform(Isometry2::translation(0., 0.)))
                .with_component(BallComponent {
                    index: 0,
                    hidden: false
                })
                .add_entity()
                .with_component(PrimitiveComponent {
                    color: [1., 1., 1., 1.],
                    shape: circle()
                })
                .with_component(Transform(Isometry2::translation(0., 0.)))
                .with_component(BallComponent {
                    index: 1,
                    hidden: false
                })
                .build(),

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

        self
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        for system in &self.systems {
            system.render(render_ctx);
        }
    }
}