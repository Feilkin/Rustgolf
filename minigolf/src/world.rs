use crate::components::physics::{Acceleration, PhysicsEvent, Position, Velocity};
use crate::components::GolfComponents;
use mela::ecs::entity::EntityBuilder;
use mela::ecs::world::{World, WorldStorage};
use mela::ecs::{DequeStorage, Entity, VecStorage};
use std::time::Instant;

pub struct MyWorld {
    pub next_entity_id: usize,
    pub entities: Vec<Entity>,
    pub components: GolfComponents,
    pub last_physics_update: Instant,
}

impl MyWorld {
    pub fn new() -> MyWorld {
        MyWorld {
            next_entity_id: 0,
            entities: Vec::new(),
            components: GolfComponents::default(),
            last_physics_update: Instant::now(),
        }
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

// TODO: get rid of these
impl WorldStorage<Position> for MyWorld {
    type Storage = VecStorage<Position>;

    fn storage(&self) -> &Self::Storage {
        &self.components.positions
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.positions
    }
}

impl WorldStorage<Velocity> for MyWorld {
    type Storage = VecStorage<Velocity>;

    fn storage(&self) -> &Self::Storage {
        &self.components.velocities
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.velocities
    }
}

impl WorldStorage<Acceleration> for MyWorld {
    type Storage = VecStorage<Acceleration>;

    fn storage(&self) -> &Self::Storage {
        &self.components.accelerations
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.accelerations
    }
}

impl WorldStorage<PhysicsEvent> for MyWorld {
    type Storage = DequeStorage<PhysicsEvent>;

    fn storage(&self) -> &Self::Storage {
        &self.components.physics_events
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.physics_events
    }
}
