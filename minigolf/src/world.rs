use std::time::Instant;

use mela::ecs::entity::EntityBuilder;
use mela::ecs::world::{World, WorldStorage};
use mela::ecs::{DequeStorage, Entity, VecStorage, Component};
use mela::ecs::system::physics::PhysicsWorld;
use crate::components::Components;
use mela::nphysics::math::Vector;
use std::collections::HashMap;
use std::any::{TypeId, Any};
use std::fmt::Debug;
use std::cell::RefCell;

pub struct MyWorld {
    pub next_entity_id: usize,
    pub entities: Vec<Entity>,
    pub components: HashMap<TypeId, Box<dyn Any>>,
    pub physics_world: PhysicsWorld<f32>,
}

impl MyWorld {
    pub fn new() -> MyWorld {
        MyWorld {
            next_entity_id: 0,
            entities: Vec::new(),
            components: HashMap::default(),
            physics_world: PhysicsWorld::new(Vector::new(0., 0.))
        }
    }

    pub fn register<C: Component + Any + Debug>(&mut self) -> () {
        self.components.insert(TypeId::of::<C>(), Box::new(VecStorage::<C>::new()));
    }
}

impl World for MyWorld {
    fn entities(&self) -> &[Entity] {
        &self.entities
    }

    fn add_entity(self) -> EntityBuilder<Self> {
        let MyWorld {
            next_entity_id,
            mut entities,
            ..
        } = self;

        let new_entity = next_entity_id.into();
        entities.push(new_entity);

        EntityBuilder::new(
            new_entity,
            MyWorld {
                next_entity_id: next_entity_id + 1,
                entities,
                ..self
            },
        )
    }

    fn remove_entity(self, entity: Entity) -> Self {
        let MyWorld { mut entities, .. } = self;

        entities.retain(|e| *e != entity);

        MyWorld { entities, ..self }
    }

    fn remove_dead(self) -> Self {
        let MyWorld { mut entities, .. } = self;

        entities.retain(|e| !e.is_dead());

        MyWorld { entities, ..self }
    }
}

impl<C: Component + Any + Debug> WorldStorage<C> for MyWorld {
    type Storage = VecStorage<C>;

    fn storage<'s, 'w: 's>(&'w self) -> &'s Self::Storage {
        self.components
            .get(&TypeId::of::<C>())
            .unwrap()
            .downcast_ref()
            .unwrap()
    }
}