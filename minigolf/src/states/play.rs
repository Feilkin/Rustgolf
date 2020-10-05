//! play :)

use crate::physics::{Ball, BallComponent, PhysicsAnimator, PhysicsBody, Snapshot, Wall};
use crate::player::{LineDrawer, PlayerController, PlayerInput, WallComponent};
use crate::world::MyWorld;
use mela::asset::tilemap::{Orthogonal, Tilemap};
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::component::Transform;
use mela::ecs::system::SystemCaller;
use mela::ecs::world::World;
use mela::game::IoState;
use mela::gfx::primitives::{PrimitiveComponent, PrimitiveShape};
use mela::gfx::RenderContext;
use mela::lyon;
use mela::lyon::algorithms::path::Path;
use mela::nphysics::ncollide2d::na::{Isometry2, Point2, Vector2};
use mela::state::State;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct Play {
    world: MyWorld,
    systems: Vec<Box<dyn SystemCaller<MyWorld>>>,
}

impl Play {
    pub fn new() -> Play {
        let mut timer = Rc::new(RefCell::new(Duration::new(0, 0)));
        let mut snapshots = Vec::new();
        let walls = Rc::new(RefCell::new(vec![
            Wall {
                start: Point2::new(4., 4.),
                end: Point2::new(4., 716.),
            },
            Wall {
                start: Point2::new(4., 716.),
                end: Point2::new(1276., 716.),
            },
            Wall {
                start: Point2::new(1276., 716.),
                end: Point2::new(1276., 4.),
            },
            Wall {
                start: Point2::new(1276., 4.),
                end: Point2::new(4., 4.),
            },
            Wall {
                start: Point2::new(63.0, 374.0),

                end: Point2::new(383.0, 689.0),
            },
            Wall {
                start: Point2::new(383.0, 689.5),
                end: Point2::new(826.0, 688.5),
            },
            Wall {
                start: Point2::new(826.0, 689.0),
                end: Point2::new(1201.0, 314.0),
            },
            Wall {
                start: Point2::new(1201.0, 313.0),
                end: Point2::new(920.0, 32.0),
            },
            Wall {
                start: Point2::new(920.0, 32.0),
                end: Point2::new(679.0, 273.0),
            },
            Wall {
                start: Point2::new(679.0, 274.0),
                end: Point2::new(861.0, 456.0),
            },
            Wall {
                start: Point2::new(861.0, 456.0),
                end: Point2::new(800.0, 517.0),
            },
            Wall {
                start: Point2::new(799.0, 517.0),
                end: Point2::new(722.0, 440.0),
            },
            Wall {
                start: Point2::new(722.0, 440.0),
                end: Point2::new(571.0, 591.0),
            },
            Wall {
                start: Point2::new(570.0, 591.0),
                end: Point2::new(481.0, 502.0),
            },
            Wall {
                start: Point2::new(481.0, 502.0),
                end: Point2::new(648.0, 335.0),
            },
            Wall {
                start: Point2::new(648.0, 335.0),
                end: Point2::new(378.0, 65.0),
            },
            Wall {
                start: Point2::new(377.0, 65.0),
                end: Point2::new(68.0, 374.0),
            },
        ]));

        let mut seed = Snapshot::new(Vec::new(), Rc::clone(&walls));

        let mut world = MyWorld::new()
            .register::<BallComponent>()
            .register::<Transform<f64>>()
            .register::<PrimitiveComponent>()
            .register::<PlayerController>()
            .register::<WallComponent>();

        world = world
            .add_entity()
            .with_component(Transform(Isometry2::translation(0., 0.)))
            .with_component(WallComponent {})
            .with_component(PrimitiveComponent {
                color: [0., 0.2, 1., 0.],
                shape: PrimitiveShape::Path(Path::new()),
            })
            .build();

        for i in 0..8 {
            let f = i as f64;
            let k = f % 2.;
            let x = 300. + f % 3. * 60.;
            let y = 200. + (f / 3.).floor() * 60. + k * 10.;

            seed.balls.push(PhysicsBody {
                body: Ball { radius: 21.335 },
                position: Point2::new(x, y),
                velocity: Vector2::new(0., 0.),
                acceleration: Vector2::new(0., 0.),
            });

            let mut entity = world
                .add_entity()
                .with_component(PrimitiveComponent {
                    color: if i == 0 {
                        [0., 1., 1., 1.]
                    } else {
                        [1., 0., 1., 1.]
                    },
                    shape: PrimitiveShape::Ball(21.335, 21.335),
                })
                .with_component(Transform(Isometry2::translation(x, y)))
                .with_component(BallComponent {
                    index: i,
                    hidden: false,
                });

            if i == 0 {
                entity = entity.with_component(PlayerController {});
            }

            world = entity.build();
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
                Box::new(PhysicsAnimator::<f64>::new(
                    Rc::clone(&snapshots),
                    Rc::clone(&timer),
                )) as Box<dyn SystemCaller<MyWorld>>,
                Box::new(PlayerInput::new(Rc::clone(&timer), Rc::clone(&snapshots))),
                Box::new(LineDrawer::new(Rc::clone(&snapshots))),
                Box::new(mela::gfx::primitives::PrimitiveRenderer::new()),
            ],
        }
    }
}

impl DebugDrawable for Play {}

impl State for Play {
    type Wrapper = Self;

    fn name(&self) -> &str {
        "Play"
    }

    fn update(
        mut self,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
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
