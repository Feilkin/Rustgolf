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

pub struct EntityBuilder<'w, W: 'w + World> {
    new_entity: Entity,
    world: W,
    phantom: PhantomData<&'w W>,
}

impl<'w, W: 'w + World> EntityBuilder<'w, W> {
    pub fn new(new_entity: Entity, world: W) -> EntityBuilder<'w, W> {
        EntityBuilder {
            new_entity,
            world,
            phantom: PhantomData,
        }
    }

    /// Consumes this entity builder, returning the new world
    pub fn build(self) -> W {
        self.world
    }

    pub fn with_component<T: 'w>(self, component: T) -> EntityBuilder<'w, W>
    where
        T: Component,
        W: ComponentStorage<'w, T>,
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
            let mut writer = world.write::<T>();
            WriteAccess::set(&mut writer, new_entity, component);
        }

        EntityBuilder {
            new_entity,
            world,
            ..self
        }
    }

    // convenient shortcute
    pub fn add_entity(self) -> EntityBuilder<'w, W> {
        self.build().add_entity()
    }
}
