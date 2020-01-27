//! World is the container thing

use crate::ecs::{entity::EntityBuilder, Entity, Component, ReadAccess, WriteAccess, ComponentStorage};

pub trait World: Sized {
    fn entities(&self) -> &[Entity];
    fn add_entity(self) -> EntityBuilder<Self>;
}

pub trait WorldStorage<C: Component>: World {
    type Storage: ComponentStorage<C>;

    fn storage(&self) -> &Self::Storage;
    fn mut_storage(&mut self) -> &mut Self::Storage;
}