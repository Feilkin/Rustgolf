//! ECS components

use crate::components::physics::{Acceleration, Position, Velocity};
use mela::ecs::{Component, Entity, ReadAccess, WriteAccess};
use std::ops::Index;

mod gfx;
pub mod physics;

#[derive(Debug, Default)]
pub struct GolfComponents {
    positions: Vec<Option<physics::Position>>,
    velocities: Vec<Option<physics::Velocity>>,
    accelerations: Vec<Option<physics::Acceleration>>,
    shapes: Vec<Option<physics::Shape>>,
}

impl GolfComponents {
    pub fn new() -> GolfComponents {
        GolfComponents::default()
    }
}

impl mela::ecs::ComponentStorage for GolfComponents {}

// TODO: get rid of these, maybe with macro?
fn vector_storage_insert<T: Component>(vector: &mut Vec<Option<T>>, entity: Entity, value: T) {
    if vector.capacity() <= *entity {
        vector.reserve(*entity - vector.capacity() + 1);
    }

    if vector.len() <= *entity {
        for _ in 1..*entity - vector.len() {
            vector.push(None);
        }
        vector.push(Some(value));
    } else {
        vector[*entity] = Some(value);
    }
}

impl WriteAccess<Position> for GolfComponents {
    fn set(&mut self, entity: Entity, value: Position) {
        vector_storage_insert(&mut self.positions, entity, value);
    }
}

impl WriteAccess<Velocity> for GolfComponents {
    fn set(&mut self, entity: Entity, value: Velocity) {
        vector_storage_insert(&mut self.velocities, entity, value);
    }
}

impl WriteAccess<Acceleration> for GolfComponents {
    fn set(&mut self, entity: Entity, value: Acceleration) {
        vector_storage_insert(&mut self.accelerations, entity, value);
    }
}

impl ReadAccess<Position> for GolfComponents {
    fn fetch(&self, entity: Entity) -> Option<&Position> {
        self.positions.get(*entity).unwrap_or(&None).as_ref()
    }

    fn enumerate(&self) -> &[Option<Position>] {
        &self.positions
    }
}

impl ReadAccess<Velocity> for GolfComponents {
    fn fetch(&self, entity: Entity) -> Option<&Velocity> {
        self.velocities.get(*entity).unwrap_or(&None).as_ref()
    }

    fn enumerate(&self) -> &[Option<Velocity>] {
        unimplemented!()
    }
}

impl ReadAccess<Acceleration> for GolfComponents {
    fn fetch(&self, entity: Entity) -> Option<&Acceleration> {
        self.accelerations.get(*entity).unwrap_or(&None).as_ref()
    }

    fn enumerate(&self) -> &[Option<Acceleration>] {
        unimplemented!()
    }
}
