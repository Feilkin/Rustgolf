use crate::api::{Player, PublicPlay, PublicStates, Putt};
use crate::physics::{BallComponent, Snapshot, Wall};
use crate::world::MyWorld;
use mela::debug::DebugContext;
use mela::ecs::component::Transform;
use mela::ecs::system::{Read, Write};
use mela::ecs::{Component, System};
use mela::game::IoState;
use mela::gfx::primitives::{PrimitiveComponent, PrimitiveShape};
use mela::gfx::RenderContext;
use mela::lyon::lyon_algorithms::path::Path;
use mela::lyon::lyon_tessellation::math::Point;
use mela::nalgebra as na;
use mela::nalgebra::Vector2;
use reqwest::blocking::Client;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PlayerController {}

impl Component for PlayerController {}

pub struct PlayerInput {
    timer: Rc<RefCell<Duration>>,
    snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
}

impl PlayerInput {
    pub fn new(
        timer: Rc<RefCell<Duration>>,
        snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
    ) -> PlayerInput {
        PlayerInput { timer, snapshots }
    }
}

#[derive(Debug)]
pub struct HitIndicator {}

impl Component for HitIndicator {}

impl System<MyWorld> for PlayerInput {
    type SystemData<'a> = (
        Read<'a, PlayerController>,
        Read<'a, BallComponent>,
        Read<'a, Transform<f64>>,
        Read<'a, HitIndicator>,
        Write<'a, PrimitiveComponent>,
    );

    fn name(&self) -> &'static str {
        "PlayerInput"
    }

    fn update<'f>(
        &mut self,
        (controller, balls, transforms, indicators, mut primitives): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> () {
        let (entity, _) = controller.iter().next().unwrap();
        let current_time = self.timer.borrow();

        let ball = balls.fetch(entity).unwrap();
        let transform = transforms.fetch(entity).unwrap().clone();

        let impulse = Vector2::new(
            io_state.mouse_position[0] as f64,
            io_state.mouse_position[1] as f64,
        ) - &transform.0.translation.vector;

        let (indicator_entity, _) = indicators.iter().next().unwrap();
        let (_, indicator_prim) = primitives
            .iter_mut()
            .find(|(e, _)| *e == indicator_entity)
            .unwrap();

        let start_point_vec = &transform.0.translation.vector + impulse.normalize() * 30.;
        let start_point = Point::new(start_point_vec.x as f32, start_point_vec.y as f32);
        let end_point = Point::new(io_state.mouse_position[0], io_state.mouse_position[1]);

        let mut path = Path::builder();
        path.move_to(start_point);
        path.line_to(end_point);

        indicator_prim.shape = PrimitiveShape::Path(path.build());

        if io_state.mouse_buttons[0] {
            if let Some(snapshot_index) = {
                let snapshots = (*self.snapshots).borrow();
                let mut found = None;
                for (i, snapshot) in snapshots.iter().enumerate() {
                    if snapshot.end_time >= *current_time {
                        found = Some(i);
                        break;
                    }
                }

                found
            } {
                let mut snapshots = self.snapshots.borrow_mut();
                let mut current_snapshot = &mut snapshots[snapshot_index];
                current_snapshot.end_time = current_time.clone();
                let mut new = current_snapshot
                    .advance_to((*current_time - current_snapshot.start_time).as_secs_f64());
                new.ignore_collisions = current_snapshot.ignore_collisions.clone();

                new.balls[ball.index].velocity = impulse;

                let mut new_snapshots: Vec<Snapshot<f64>> =
                    snapshots[..=snapshot_index].iter().cloned().collect();
                new_snapshots.push(new);

                let mut seed_index = snapshot_index + 1;

                while let Some(next) = new_snapshots[seed_index].next_snapshot() {
                    new_snapshots.push(next);
                    seed_index += 1;
                }

                *snapshots = new_snapshots;
            }
        }
    }
}

#[derive(Debug)]
pub struct WallComponent {}

impl Component for WallComponent {}

pub struct LineDrawer {
    snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
    added_wall: bool,
}

impl LineDrawer {
    pub fn new(snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>) -> LineDrawer {
        LineDrawer {
            snapshots,
            added_wall: false,
        }
    }
}

impl System<MyWorld> for LineDrawer {
    type SystemData<'a> = (Read<'a, WallComponent>, Write<'a, PrimitiveComponent>);

    fn name(&self) -> &'static str {
        "LineDrawer"
    }

    fn update<'f>(
        &mut self,
        (wall_components, mut primitives): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> () {
        let snapshots = self.snapshots.borrow_mut();
        let mut walls = snapshots.first().unwrap().walls.borrow_mut();

        let (wall_entity, _) = wall_components.iter().next().unwrap();
        let (_, primitive) = primitives
            .iter_mut()
            .find(|(e, _)| *e == wall_entity)
            .unwrap();

        let mut path_builder = Path::builder();

        for wall in walls.iter() {
            let (sx, sy) = (wall.start.x, wall.start.y);
            let (ex, ey) = (wall.end.x, wall.end.y);

            path_builder.move_to(Point::new(sx as f32, sy as f32));
            path_builder.line_to(Point::new(ex as f32, ey as f32));
        }

        primitive.shape = PrimitiveShape::Path(path_builder.build());
    }
}

pub struct MultiplayerInput {
    timer: Rc<RefCell<Duration>>,
    snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
    initial_snapshot: Snapshot<f64>,
    click_cooldown: Duration,
    client: Rc<Client>,
    endpoint: String,
    game_id: usize,
    uuid: Uuid,
}

impl MultiplayerInput {
    pub fn new(
        timer: Rc<RefCell<Duration>>,
        snapshots: Rc<RefCell<Vec<Snapshot<f64>>>>,
        initial_snapshot: Snapshot<f64>,
        client: Rc<Client>,
        endpoint: String,
        game_id: usize,
        uuid: Uuid,
    ) -> MultiplayerInput {
        MultiplayerInput {
            timer,
            snapshots,
            initial_snapshot,
            client,
            endpoint,
            game_id,
            click_cooldown: Duration::new(0, 0),
            uuid,
        }
    }
}

impl System<MyWorld> for MultiplayerInput {
    type SystemData<'a> = (
        Read<'a, PlayerController>,
        Read<'a, BallComponent>,
        Read<'a, Transform<f64>>,
        Read<'a, HitIndicator>,
        Write<'a, PrimitiveComponent>,
    );

    fn name(&self) -> &'static str {
        "MultiplayerInput"
    }

    fn update<'f>(
        &mut self,
        (controller, balls, transforms, indicators, mut primitives): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> () {
        let (entity, _) = controller.iter().next().unwrap();
        let current_time = self.timer.borrow();

        let ball = balls.fetch(entity).unwrap();
        let transform = transforms.fetch(entity).unwrap().clone();

        self.click_cooldown += delta;

        let (indicator_entity, _) = indicators.iter().next().unwrap();
        let (_, indicator_prim) = primitives
            .iter_mut()
            .find(|(e, _)| *e == indicator_entity)
            .unwrap();

        if self.click_cooldown >= Duration::new(1, 0) {
            let impulse = Vector2::new(
                io_state.mouse_position[0] as f64,
                io_state.mouse_position[1] as f64,
            ) - &transform.0.translation.vector;

            let start_point_vec = &transform.0.translation.vector + impulse.normalize() * 30.;
            let start_point = Point::new(start_point_vec.x as f32, start_point_vec.y as f32);
            let end_point = Point::new(io_state.mouse_position[0], io_state.mouse_position[1]);

            let mut path = Path::builder();
            path.move_to(start_point);
            path.line_to(end_point);

            indicator_prim.shape = PrimitiveShape::Path(path.build());

            if io_state.mouse_buttons[0] {
                self.click_cooldown = Duration::new(0, 0);

                let time = self.timer.borrow().clone();

                let resp = self
                    .client
                    .put(&self.endpoint)
                    .json::<Putt>(&Putt {
                        id: self.game_id,
                        player: Player {
                            uuid: self.uuid.to_hyphenated().to_string(),
                        },
                        time,
                        impulse: impulse.into(),
                    })
                    .send()
                    .unwrap()
                    .json::<PublicStates>();

                match resp {
                    Ok(PublicStates::Play(mut state)) => {
                        let mut snapshots = vec![self.initial_snapshot.clone()];

                        state.puts.sort_by(|a, b| a.time.cmp(&b.time));

                        for putt in state.puts {
                            let (i, snapshot) = snapshots
                                .iter_mut()
                                .enumerate()
                                .find(|(_, s)| s.end_time > putt.time)
                                .unwrap();

                            snapshot.end_time = putt.time.clone();
                            let mut new = snapshot
                                .advance_to((putt.time - snapshot.start_time).as_secs_f64());

                            new.ignore_collisions = snapshot.ignore_collisions.clone();

                            new.balls[putt.player.id].velocity = putt.impulse.into();

                            let mut new_snapshots: Vec<Snapshot<f64>> =
                                snapshots[..=i].iter().cloned().collect();
                            new_snapshots.push(new);

                            let mut seed_index = i + 1;

                            while let Some(next) = new_snapshots[seed_index].next_snapshot() {
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
                    _ => (),
                }
            }
        } else {
            indicator_prim.shape = PrimitiveShape::Path(Path::new())
        }
    }
}
