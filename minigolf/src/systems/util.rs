use crate::world::MyWorld;
use mela::ecs::System;
use mela::profiler;
use mela::profiler::{PopTag, PushTag};
use std::time::Duration;

pub struct FixedInterval<S: System<MyWorld>> {
    inner: S,
    delta_buffer: Duration,
    tick_interval: Duration,
}

impl<S: System<MyWorld>> FixedInterval<S> {
    pub fn wrap(system: S, interval: Duration) -> FixedInterval<S> {
        FixedInterval {
            inner: system,
            tick_interval: interval,
            delta_buffer: Duration::new(0, 0),
        }
    }
}

impl<S: System<MyWorld>> System<MyWorld> for FixedInterval<S> {
    fn name(&self) -> &'static str {
        "FixedInterval"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        mut profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (MyWorld, profiler::OpenTagTree<'f>) {
        self.delta_buffer += delta;

        let mut world = world;

        while self.delta_buffer >= self.tick_interval {
            let mut child_tag = profiler_tag.push_tag(self.inner.name(), [0.5, 0.2, 0.3, 1.0]);
            let (w, child_tag) = self.inner.update(self.tick_interval, world, child_tag);

            world = w;
            profiler_tag = child_tag.pop_tag();
            self.delta_buffer -= self.tick_interval;

            break;
        }

        (world, profiler_tag)
    }
}

pub struct SystemGroup {
    group: Vec<Box<dyn System<MyWorld>>>,
}

impl SystemGroup {
    pub fn new(group: Vec<Box<dyn System<MyWorld>>>) -> SystemGroup {
        SystemGroup { group }
    }
}

impl System<MyWorld> for SystemGroup {
    fn name(&self) -> &'static str {
        "Group"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        mut profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (MyWorld, profiler::OpenTagTree<'f>) {
        let mut world = world;

        for system in self.group.iter_mut() {
            let mut child_tag = profiler_tag.push_tag(system.name(), [0.5, 0.2, 0.3, 1.0]);
            let (w, child_tag) = system.update(delta, world, child_tag);
            profiler_tag = child_tag.pop_tag();
            world = w;
        }

        (world, profiler_tag)
    }
}
