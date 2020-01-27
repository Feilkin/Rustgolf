//! Entity related stuff

use crate::ecs::world::World;
use crate::ecs::{Component, ComponentStorage, WriteAccess};
use serde::export::PhantomData;
use std::ops::Deref;

/// Entities are just ID's
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        EntityBuilder {
            new_entity,
            world,
        }
    }

    /// consumes this entity builder, returning the new world
    pub fn build(self) -> W {
        self.world
    }

    /// adds Component for the new Entity into the World
    pub fn with_component<'d, 'a, T>(self, component: T) -> EntityBuilder<W>
    where
        'd: 'a,
        T: 'd + Component,
        W: 'd + ComponentStorage<'d, 'a, T>,
    {
        let EntityBuilder {
            new_entity,
            mut world,
            ..
        } = self;

        // TODO: write component
        //let DefaultWorld { mut components, .. } = self.world;
        //ComponentStorage::write(&mut components, self.new_entity, component);
        {
            let mut writer = world.write();
            WriteAccess::set(&mut writer, new_entity, component);

            // writer dropped here
        }

        EntityBuilder {
            new_entity,
            world,
            ..self
        }
        // compiler complains that world is still borrowed here?
    }

    /// builds the entity, immediately calling add_entity on the underlying World.
    ///
    /// Allows chaining of `add_entity()` calls.
    pub fn add_entity(self) -> EntityBuilder<W> {
        self.build().add_entity()
    }
}
