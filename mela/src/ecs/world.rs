//! World is the container thing

use crate::ecs::{entity::EntityBuilder, Entity};

pub trait World: Sized {
    fn entities(&self) -> &[Entity];
    fn add_entity(self) -> EntityBuilder<Self>;
}
