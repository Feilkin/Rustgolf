//! Entity related stuff

use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage, WriteAccess};
use serde::export::PhantomData;
use std::ops::Deref;

/// Entities are just ID's
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Adds new entity, and possible components, to a World.
pub struct EntityBuilder<W: World> {
    new_entity: Entity,
    world: W,
}

impl<W: World> EntityBuilder<W> {
    /// constructs a new EntityBuilder from new entity, and a World.
    /// Consumes the World.
    pub fn new(new_entity: Entity, world: W) -> EntityBuilder<W> {
        EntityBuilder { new_entity, world }
    }

    /// consumes this entity builder, returning the new world
    pub fn build(self) -> W {
        self.world
    }

    /// adds Component for the new Entity into the World
    pub fn with_component<T>(self, component: T) -> EntityBuilder<W>
    where
        T: Component,
        W: WorldStorage<T>,
    {
        let EntityBuilder {
            new_entity,
            mut world,
            ..
        } = self;

        world.mut_storage().write().set(new_entity, component);

        EntityBuilder {
            new_entity,
            world,
            ..self
        }
    }

    /// builds the entity, immediately calling add_entity on the underlying World.
    ///
    /// Allows chaining of `add_entity()` calls.
    pub fn add_entity(self) -> EntityBuilder<W> {
        self.build().add_entity()
    }
}
