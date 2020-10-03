//! Analytical physics engine

use mela::nalgebra as na;
use mela::ecs::{System, Component};
use mela::gfx::RenderContext;
use mela::game::IoState;
use std::time::{Instant, Duration};
use mela::debug::DebugContext;
use mela::ecs::system::{Read, Write};
use std::rc::Rc;
use mela::nphysics::ncollide2d::simba::scalar::RealField;
use mela::ecs::component::Transform;
use mela::nphysics::ncollide2d::na::{Point2, Similarity2, Vector2, Isometry3, Isometry2};
use mela::ecs::world::{World, WorldStorage};
use std::ops::Mul;
use std::cell::RefCell;
use std::borrow::Borrow;

#[derive(Clone, Debug)]
pub struct PhysicsBody<T, N: RealField = f64> {
    body: T,
    position: na::Point2<N>,
    velocity: na::Vector2<N>,
    acceleration: na::Vector2<N>
}

#[derive(Clone, Debug)]
pub struct Ball<N: RealField = f64> {
    radius: N
}

#[derive(Clone, Debug)]
pub struct BallComponent {
    index: usize,
    hidden: bool
}

impl Component for BallComponent {}

#[derive(Clone, Debug)]
pub struct Snapshot<N: RealField> {
    start_time: Duration,
    end_time: Duration,
    balls: Vec<PhysicsBody<Ball<N>, N>>
}

impl Snapshot<f64> {
    pub fn ball_pos(&self, ball_index: usize, t: Duration) -> (Point2<f64>, &Ball<f64>) {
        let body = &self.balls[ball_index];
        let delta = (t - self.start_time).as_secs_f64();

        let x = &body.position.x + &body.velocity.x * delta + 0.5 * &body.acceleration.x * delta.powf(2.);
        let y = &body.position.y + &body.velocity.y * delta + 0.5 * &body.acceleration.y * delta.powf(2.);

        (Point2::new(x, y), &body.body)
    }
}

pub struct PhysicsAnimator<N: RealField> {
    snapshots: Rc<RefCell<Vec<Snapshot<N>>>>,
    timer: Duration,
}

impl<N> PhysicsAnimator<N> where N: RealField {
    pub fn new(snapshots: Rc<RefCell<Vec<Snapshot<N>>>>) -> PhysicsAnimator<N> {
        PhysicsAnimator {
            snapshots,
            timer: Duration::new(0, 0)
        }
    }
}

impl<W> System<W> for PhysicsAnimator<f64> where W: World + WorldStorage<Transform<f64>> + WorldStorage<BallComponent> {
    type SystemData<'a> = (Write<'a, Transform<f64>>, Read<'a, BallComponent>);

    fn name(&self) -> &'static str {
        "PhysicsAnimator"
    }

    fn update<'f>(&mut self, (mut transforms, balls): Self::SystemData<'f>, delta: Duration, _io_state: &IoState, _render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) -> () {
        let current_time = self.timer + delta;
        if let Some(current_snapshot) = {
            let snapshots = (*self.snapshots).borrow();
            let mut found = None;
            for snapshot in &*snapshots {
                if snapshot.end_time >= current_time { found = Some(snapshot.clone()); break }
            }

            found.or_else(|| snapshots.last().cloned())
        } {
            for (entity, mut transform) in transforms.iter_mut() {
                if let Some(ball) = balls.fetch(entity) {
                    let BallComponent {
                        index, hidden
                    } = ball;

                    if !hidden {
                        let (pos, ball) = current_snapshot.ball_pos(*index, current_time);
                        transform.0 = Isometry2::translation(pos.x, pos.y);
                    }
                }
            }

            self.timer = current_time;
        }
    }
}