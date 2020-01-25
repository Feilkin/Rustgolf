//! ECS components

use crate::components::physics::{Acceleration, Position, Velocity};
use mela::ecs::{Component, Entity, ReadAccess, WriteAccess};

mod gfx;
pub mod physics;

#[derive(Debug, Default)]
pub struct GolfComponents {
    positions: Vec<(Entity, physics::Position)>,
    velocities: Vec<(Entity, physics::Velocity)>,
    accelerations: Vec<(Entity, physics::Acceleration)>,
    shapes: Vec<physics::Shape>,
}

impl GolfComponents {
    pub fn new() -> GolfComponents {
        GolfComponents::default()
    }
}

impl mela::ecs::ComponentStorage for GolfComponents {
    fn read<T>(&self) -> &[(Entity, T)]
    where
        T: Component,
        Self: ReadAccess<T>,
    {
        ReadAccess::fetch(self)
    }

    fn write<T>(&mut self, entity: Entity, value: T)
    where
        T: Component,
        Self: WriteAccess<T>,
    {
        WriteAccess::set(self, entity, value)
    }
}

// TODO: get rid of these, maybe with macro?

impl WriteAccess<Position> for GolfComponents {
    fn set(&mut self, entity: Entity, value: Position) {
        match self.positions.iter().position(|(e, _)| *e == entity) {
            Some(index) => self.positions[index] = (entity, value),
            None => self.positions.push((entity, value)),
        }
    }
}

impl WriteAccess<Velocity> for GolfComponents {
    fn set(&mut self, entity: Entity, value: Velocity) {
        match self.velocities.iter().position(|(e, _)| *e == entity) {
            Some(index) => self.velocities[index] = (entity, value),
            None => self.velocities.push((entity, value)),
        }
    }
}

impl WriteAccess<Acceleration> for GolfComponents {
    fn set(&mut self, entity: Entity, value: Acceleration) {
        match self.accelerations.iter().position(|(e, _)| *e == entity) {
            Some(index) => self.accelerations[index] = (entity, value),
            None => self.accelerations.push((entity, value)),
        }
    }
}

impl ReadAccess<Position> for GolfComponents {
    fn fetch(&self) -> &[(Entity, Position)] {
        &self.positions
    }
}

impl ReadAccess<Velocity> for GolfComponents {
    fn fetch(&self) -> &[(Entity, Velocity)] {
        &self.velocities
    }
}

impl ReadAccess<Acceleration> for GolfComponents {
    fn fetch(&self) -> &[(Entity, Acceleration)] {
        &self.accelerations
    }
}
