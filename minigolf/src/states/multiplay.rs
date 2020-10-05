use crate::api::{GetUpdate, PublicStates};
use crate::physics::{Ball, BallComponent, PhysicsAnimator, PhysicsBody, Snapshot, Wall};
use crate::player::{LineDrawer, MultiplayerInput, PlayerController, PlayerInput, WallComponent};
use crate::states::{walls, Play, Wrapper};
use crate::world::MyWorld;
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::component::Transform;
use mela::ecs::system::SystemCaller;
use mela::ecs::world::World;
use mela::ecs::System;
use mela::game::IoState;
use mela::gfx::primitives::{PrimitiveComponent, PrimitiveShape};
use mela::gfx::RenderContext;
use mela::lyon::lyon_algorithms::path::Path;
use mela::nalgebra::{Isometry2, Point2, Vector2};
use mela::state::State;
use reqwest::blocking::Client;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug)]
pub enum GameState {
    Waiting,
    Warmup,
    Play,
    GameOver(Duration),
}

pub struct Multiplay {
    game_state: Rc<RefCell<GameState>>,
    client: Rc<Client>,
    uuid: Uuid,
    own_ball_index: usize,
    world: MyWorld,
    systems: Vec<Box<dyn SystemCaller<MyWorld>>>,
}
impl Multiplay {
    pub fn new(client: Client, uuid: Uuid, game_id: usize, own_ball_index: usize) -> Multiplay {
        let player_colors = [
            [1., 0., 0.98, 1.0],
            [0.46, 0., 1., 1.0],
            [1.0, 0., 0.0, 1.0],
            [0.71, 1., 0.7, 1.],
            [0.0, 1., 0.18, 1.],
            [1.0, 0.56, 0., 1.],
            [1.0, 0.96, 0., 1.],
        ];

        let mut client = Rc::new(client);
        let mut timer = Rc::new(RefCell::new(Duration::new(0, 0)));
        let mut snapshots = Vec::new();
        let walls = walls();

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
            .add_entity()
            .with_component(Transform(Isometry2::translation(920., 300.)))
            .with_component(PrimitiveComponent {
                color: [0.3, 1., 0.3, 1.],
                shape: PrimitiveShape::Ball(100., 100.),
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
                    color: if i == own_ball_index {
                        [0., 1., 1., 1.]
                    } else if i < own_ball_index {
                        player_colors[i]
                    } else {
                        player_colors[i - 1]
                    },
                    shape: PrimitiveShape::Ball(21.335, 21.335),
                })
                .with_component(Transform(Isometry2::translation(x, y)))
                .with_component(BallComponent {
                    index: i,
                    hidden: false,
                });

            if i == own_ball_index {
                entity = entity.with_component(PlayerController {});
            }

            world = entity.build();
        }

        let initial_snapshot = seed.clone();

        snapshots.push(seed);

        let mut seed_index = 0;
        while let Some(next) = snapshots[seed_index].next_snapshot() {
            snapshots.push(next);
            seed_index += 1;
        }

        let snapshots = Rc::new(RefCell::new(snapshots));
        let game_state = Rc::new(RefCell::new(GameState::Waiting));

        Multiplay {
            systems: vec![
                Box::new(PhysicsAnimator::<f64>::new(
                    Rc::clone(&snapshots),
                    Rc::clone(&timer),
                )) as Box<dyn SystemCaller<MyWorld>>,
                Box::new(LineDrawer::new(Rc::clone(&snapshots))),
                Box::new(mela::gfx::primitives::PrimitiveRenderer::new()),
                Box::new(PollerSystem::new(
                    "http://minigolf.srvrs.eu".to_owned(),
                    game_id,
                    Rc::clone(&game_state),
                    Rc::clone(&client),
                    uuid.clone(),
                    Rc::clone(&snapshots),
                    initial_snapshot,
                )),
                Box::new(MultiplayerInput::new(
                    Rc::clone(&timer),
                    Rc::clone(&snapshots),
                    Rc::clone(&client),
                    "http://minigolf.srvrs.eu".to_owned(),
                    game_id,
                    uuid.clone(),
                )),
            ],
            game_state,
            client,
            uuid,
            own_ball_index,
            world,
        }
    }
}

impl DebugDrawable for Multiplay {}

impl State for Multiplay {
    type Wrapper = Wrapper;

    fn name(&self) -> &str {
        "Multiplay"
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

        {
            let game_state = self.game_state.borrow();

            match *game_state {
                GameState::Waiting => ui.text(im_str!("Waiting for another player to join...")),
                GameState::Warmup => ui.text(im_str!("Waiting for additional players...")),
                GameState::Play => {
                    ui.text(im_str!("Play!"));
                    ui.text(im_str!("Try to get your ball to end\nup in the green circle when\ntime runs out.\nYou have 4 shots."));
                }
                GameState::GameOver(elapsed) => {
                    if elapsed >= Duration::new(25, 0) {
                        return Wrapper::Play(Play::new());
                    }

                    ui.text(im_str!("Game Over!"))
                }
            }
        }

        Wrapper::Multiplay(self)
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        for system in &self.systems {
            system.render(render_ctx);
        }
    }
}

struct PollerSystem {
    client: Rc<Client>,
    game_id: usize,
    uuid: Uuid,
    endpoint: String,
    snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
    game_state: Rc<RefCell<GameState>>,
    update_timer: Duration,
    initial_snapshot: Snapshot<f64>,
    last_put_count: usize,
}

impl PollerSystem {
    pub fn new(
        endpoint: String,
        game_id: usize,
        game_state: Rc<RefCell<GameState>>,
        client: Rc<Client>,
        uuid: Uuid,
        snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
        initial_snapshot: Snapshot<f64>,
    ) -> PollerSystem {
        PollerSystem {
            game_id,
            endpoint,
            game_state,
            client,
            uuid,
            snapshots,
            update_timer: Duration::new(0, 0),
            initial_snapshot,
            last_put_count: 0,
        }
    }
}

impl System<MyWorld> for PollerSystem {
    type SystemData<'f> = ();

    fn name(&self) -> &'static str {
        "PollerSystem"
    }

    fn update<'f>(
        &mut self,
        data: Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> () {
        let poll_rate = Duration::new(1, 0);

        self.update_timer += delta;

        if self.update_timer >= poll_rate {
            self.update_timer = Duration::new(0, 0);

            match self
                .client
                .get(&self.endpoint)
                .json(&GetUpdate { id: self.game_id })
                .send()
            {
                Ok(resp) => {
                    if resp.status() == 200 {
                        let state = resp.json::<PublicStates>().unwrap();
                        let mut our_state = self.game_state.borrow_mut();

                        *our_state = match state {
                            PublicStates::WaitingForPlayers(_) => GameState::Waiting,
                            PublicStates::Warmup(_) => GameState::Warmup,
                            PublicStates::Play(state) => {
                                if state.puts.len() != self.last_put_count {
                                    self.last_put_count = state.puts.len();

                                    let mut snapshots = vec![self.initial_snapshot.clone()];

                                    for putt in state.puts {
                                        let (i, snapshot) = snapshots
                                            .iter_mut()
                                            .enumerate()
                                            .find(|(_, s)| s.end_time > putt.time)
                                            .unwrap();

                                        snapshot.end_time = putt.time.clone();
                                        let mut new = snapshot.advance_to(
                                            (putt.time - snapshot.start_time).as_secs_f64(),
                                        );

                                        new.ignore_collisions = snapshot.ignore_collisions.clone();

                                        new.balls[putt.player.id].velocity = putt.impulse.into();

                                        let mut new_snapshots: Vec<Snapshot<f64>> =
                                            snapshots[..=i].iter().cloned().collect();
                                        new_snapshots.push(new);

                                        let mut seed_index = i + 1;

                                        while let Some(next) =
                                            new_snapshots[seed_index].next_snapshot()
                                        {
                                            new_snapshots.push(next);
                                            seed_index += 1;
                                        }

                                        snapshots = new_snapshots;
                                    }

                                    {
                                        let mut lock = self.snapshots.borrow_mut();
                                        *lock = snapshots;
                                    }
                                }

                                GameState::Play
                            }
                            PublicStates::GameOver(s) => GameState::GameOver(s.time),
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }
}
