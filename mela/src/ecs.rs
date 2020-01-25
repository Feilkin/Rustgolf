//! my own entity component system

use std::net::Shutdown::Write;
use std::ops::Deref;

/// Entities are just ID's
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity(usize);

impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct EntityBuilder<C: ComponentStorage> {
    new_entity: Entity,
    world: World<C>,
}

impl<C: ComponentStorage> EntityBuilder<C> {
    pub fn new(new_entity: Entity, world: World<C>) -> EntityBuilder<C> {
        EntityBuilder { new_entity, world }
    }

    /// Consumes this entity builder, returning the new world
    pub fn build(self) -> World<C> {
        self.world
    }

    pub fn with_component<T>(self, component: T) -> EntityBuilder<C>
    where
        T: Component,
        C: WriteAccess<T>,
    {
        let World { mut components, .. } = self.world;

        ComponentStorage::write(&mut components, self.new_entity, component);

        EntityBuilder {
            world: World {
                components,
                ..self.world
            },
            ..self
        }
    }

    // convenient shortcute
    pub fn add_entity(self) -> EntityBuilder<C> {
        self.build().add_entity()
    }
}

pub struct World<C: ComponentStorage> {
    pub next_entity_id: usize,
    pub entities: Vec<Entity>,
    pub components: C,
}

impl<C: ComponentStorage> World<C> {
    pub fn new(components: C) -> World<C> {
        World {
            next_entity_id: 0,
            entities: Vec::new(),
            components,
        }
    }

    pub fn add_entity(self) -> EntityBuilder<C> {
        let World {
            next_entity_id,
            mut entities,
            ..
        } = self;

        let new_entity = Entity(next_entity_id);
        entities.push(new_entity);

        EntityBuilder::new(
            new_entity,
            World {
                next_entity_id: next_entity_id + 1,
                entities,
                ..self
            },
        )
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}

pub trait ComponentStorage {
    /// returns slice of components, indexed by entity
    fn read<T>(&self, entity: Entity) -> Option<&T>
    where
        T: Component,
        Self: ReadAccess<T>,
    {
        self.fetch(entity)
    }

    fn fetch_all<T>(&self) -> &[Option<T>]
    where
        T: Component,
        Self: ReadAccess<T>,
    {
        ReadAccess::enumerate(self)
    }

    /// writes new component value for entity
    fn write<T>(&mut self, entity: Entity, value: T)
    where
        T: Component,
        Self: WriteAccess<T>,
    {
        WriteAccess::set(self, entity, value);
    }
}

pub trait Component: Sized + Send + Sync {}

pub trait ReadAccess<C: Component> {
    fn fetch(&self, entity: Entity) -> Option<&C>;
    fn enumerate(&self) -> &[Option<C>];
}

pub trait WriteAccess<C: Component> {
    fn set(&mut self, entity: Entity, value: C);
}
