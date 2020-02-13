//! entity component Systems

use crate::ecs::world::World;
use std::time::Duration;
use crate::profiler::Profiler;
use crate::profiler;

pub trait System<W: World> {
    fn update(&mut self, delta: Duration, world: W) -> W;
}
