//! ECS components

use crate::components::physics::{Acceleration, PhysicsEvent, Position, Velocity};
use mela::ecs::{Component, DequeStorage, Entity, ReadAccess, VecStorage, WriteAccess};
use std::collections::VecDeque;
use std::iter::{Enumerate, FilterMap};
use std::ops::Index;
use std::slice::Iter;

mod gfx;
pub mod physics;

#[derive(Debug, Default)]
pub struct GolfComponents {
    pub positions: VecStorage<Position>,
    pub velocities: VecStorage<Velocity>,
    pub accelerations: VecStorage<Acceleration>,
    pub physics_events: DequeStorage<PhysicsEvent>,
}
