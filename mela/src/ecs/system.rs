//! entity component Systems

use crate::ecs::world::World;
use std::time::Duration;

pub trait System<W: World> {
    fn update(&mut self, delta: Duration, world: W) -> W;
}