//! entity component Systems

use crate::ecs::world::World;
use crate::profiler;
use crate::profiler::{OpenTagTree, Profiler};
use std::time::Duration;

pub trait System<W: World> {
    fn name(&self) -> &'static str;
    fn update<'f>(
        &mut self,
        delta: Duration,
        world: W,
        profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (W, OpenTagTree<'f>);
}
