use mela::ecs::System;
use std::time::Duration;
use crate::world::MyWorld;

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
    fn update(&mut self, delta: Duration, world: MyWorld) -> MyWorld {
        self.delta_buffer += delta;
        let mut world = world;

        while self.delta_buffer >= self.tick_interval {
            world = self.inner.update(self.tick_interval, world);

            self.delta_buffer -= self.tick_interval;
        }

        world
    }
}

pub struct SystemGroup {
    group: Vec<Box<dyn System<MyWorld>>>,
}

impl SystemGroup {
    pub fn new(group: Vec<Box<dyn System<MyWorld>>>) -> SystemGroup {
        SystemGroup {
            group
        }
    }
}

impl System<MyWorld> for SystemGroup {
    fn update(&mut self, delta: Duration, world: MyWorld) -> MyWorld {
        let mut world = world;

        for system in self.group.iter_mut() {
            world = system.update(delta, world);
        }

        world
    }
}
