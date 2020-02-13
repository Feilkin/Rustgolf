use crate::components::physics::{Acceleration, PhysicsEvent, Position, Velocity};
use mela::ecs::{ComponentStorage, Entity, ReadAccess, RwAccess, System, WriteAccess};
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::world::MyWorld;
use mela::ecs::world::World;
use mela::nalgebra::Vector2;
use mela::profiler;
use mela::profiler::OpenTagTreeRoot;

pub struct MoveSystem {}

impl MoveSystem {
    pub fn new() -> MoveSystem {
        MoveSystem {}
    }

    fn move_entity(
        delta: Duration,
        position: &Position,
        velocity: &Velocity,
        acceleration: &Acceleration,
    ) -> (Position, Velocity) {
        // I read somewhere that this gives better results than just updating velocity completely
        let half_of_velocity_delta = **acceleration * 0.5 * delta.as_secs_f32();
        let velocity = **velocity + half_of_velocity_delta;
        let mut position = **position + velocity * delta.as_secs_f32();

        let (screen_width, screen_height) = (1920., 1080.);

        if position.x > screen_width + 8. {
            position.x = -8.
        } else if position.x < -8. {
            position.x = screen_width + 8.
        }

        if position.y > screen_height + 8. {
            position.y = -8.
        } else if position.y < -8. {
            position.y = screen_height + 8.
        }

        (
            position.into(),
            (velocity * 0.998 + half_of_velocity_delta).into(),
        )
    }
}

impl System<MyWorld> for MoveSystem {
    fn update(&mut self, delta: Duration, world: MyWorld) -> MyWorld {
        let MyWorld {
            entities,
            mut components,
            ..
        } = world;

        for entity in &entities {
            match (
                components.positions.read().fetch(*entity),
                components.velocities.read().fetch(*entity),
                components.accelerations.read().fetch(*entity),
            ) {
                (Some(p), Some(v), Some(a)) => {
                    let (position, velocity) = MoveSystem::move_entity(delta, p, v, a);
                    components.positions.write().set(*entity, position);
                    components.velocities.write().set(*entity, velocity);
                    components
                        .accelerations
                        .write()
                        .set(*entity, Acceleration::new(0., 0.));
                }
                _ => (),
            }
        }

        MyWorld {
            last_physics_update: Instant::now(),
            entities,
            components,
            ..world
        }
    }
}

pub struct CollisionGenerator {
    collision_set: HashSet<(Entity, Entity)>,
}

impl CollisionGenerator {
    pub fn new() -> CollisionGenerator {
        CollisionGenerator {
            collision_set: HashSet::new(),
        }
    }

    fn collide_entities(
        &mut self,
        delta: Duration,
        entity: &Entity,
        position: &Position,
        velocity: &Velocity,
        others: &mut dyn Iterator<Item = (Entity, &Position)>,
    ) -> Vec<PhysicsEvent> {
        use mela::nalgebra::Isometry2;
        use mela::ncollide2d::query;
        use mela::ncollide2d::query::Proximity;
        use mela::ncollide2d::shape::Ball;

        let mut events = Vec::new();

        // TODO: get rid of hard coded bodies
        let body = Ball::new(8f32);
        let self_isometry = Isometry2::new(position.coords, 0.);

        for (other_entity, other_position) in others {
            if other_entity == *entity {
                continue;
            }

            // avoid creating 2 events for 1 collision
            if self.collision_set.contains(&(other_entity, *entity)) {
                continue;
            }

            let other_body = Ball::new(8f32);
            let other_isometry = Isometry2::new(other_position.coords, 0.);

            match query::proximity(&self_isometry, &body, &other_isometry, &other_body, 0.) {
                Proximity::Intersecting => {
                    // collision, figure where and how
                    // TODO: figure out how to use prediction?
                    let contact =
                        query::contact(&self_isometry, &body, &other_isometry, &other_body, 0.1);

                    if let Some(contact) = contact {
                        let toi = query::time_of_impact(
                            &self_isometry,
                            &velocity,
                            &body,
                            &other_isometry,
                            &Vector2::new(0., 0.), // TODO: use actual speeds here?? idk
                            &other_body,
                            delta.as_secs_f32(),
                            0.,
                        )
                        .expect("should collide but didn't??");

                        events.push(PhysicsEvent::Collision {
                            cause: entity.clone(),
                            other: other_entity,
                            contact,
                            toi: toi.toi,
                        });

                        self.collision_set.insert((*entity, other_entity));
                    }
                }
                _ => (), // no collision
            }
        }

        events
    }
}

impl System<MyWorld> for CollisionGenerator {
    fn update(&mut self, delta: Duration, world: MyWorld) -> MyWorld {
        let MyWorld {
            entities,
            mut components,
            ..
        } = world;

        //let mut current_collisions = HashSet::new();

        components.physics_events.write().clear();

        let mut new_events = Vec::new();

        for entity in &entities {
            match (
                components.positions.read().fetch(*entity),
                components.velocities.read().fetch(*entity),
            ) {
                (Some(p), Some(v)) => {
                    let collisions = self.collide_entities(
                        delta,
                        entity,
                        p,
                        v,
                        &mut components.positions.read().iter(),
                    );
                    new_events.extend(collisions);
                }
                _ => (),
            }
        }

        let mut new_world = MyWorld {
            entities,
            components,
            ..world
        };

        for event in new_events {
            new_world = new_world.add_entity().with_component(event).build()
        }

        new_world
    }
}

pub struct CollisionResolver {}

impl CollisionResolver {
    pub fn new() -> CollisionResolver {
        CollisionResolver {}
    }

    fn resolve_collision<
        'r,
        V: RwAccess<'r, Velocity>,
        P: ReadAccess<'r, Position>,
        A: WriteAccess<'r, Acceleration>,
    >(
        collision: &PhysicsEvent,
        mut velocities: V,
        positions: P,
        mut accelerations: A,
    ) {
        match collision {
            PhysicsEvent::Collision {
                cause,
                other,
                contact,
                ..
            } => {
                let (cause, other) = (*cause, *other);
                let cause_velocity = **velocities.fetch(cause).unwrap();
                let other_velocity = **velocities.fetch(other).unwrap();
                let cause_position = **positions.fetch(cause).unwrap();
                let other_position = **positions.fetch(other).unwrap();

                let pos_diff = &cause_position - &other_position;
                let pos_diff2 = -&pos_diff;

                // TODO: implement mass
                let (cause_mass, other_mass) = (1f32, 1f32);

                let new_cause_velocity = &cause_velocity
                    - ((2. * other_mass) / (cause_mass + other_mass))
                        * ((&cause_velocity - &other_velocity).dot(&pos_diff)
                            / (pos_diff.norm().powf(2.0)))
                        * pos_diff
                        * 0.97;

                let new_other_velocity = &other_velocity
                    - ((2. * cause_mass) / (cause_mass + other_mass))
                        * ((&other_velocity - &cause_velocity).dot(&pos_diff2)
                            / (pos_diff2.norm().powf(2.0)))
                        * pos_diff2
                        * 0.97;

                velocities.set(cause, new_cause_velocity.into());
                velocities.set(other, new_other_velocity.into());

                accelerations.set(
                    cause,
                    (pos_diff.normalize() * (contact.depth.powi(2) * 50.)).into(),
                );
                accelerations.set(
                    other,
                    (pos_diff2.normalize() * (contact.depth.powi(2) * 50.)).into(),
                );
            }
            _ => (),
        }
    }
}

impl System<MyWorld> for CollisionResolver {
    fn update(&mut self, delta: Duration, world: MyWorld) -> MyWorld {
        let MyWorld {
            mut components,
            mut entities,
            ..
        } = world;

        for (event_entity, event) in components.physics_events.read().iter() {
            CollisionResolver::resolve_collision(
                event,
                components.velocities.write(),
                components.positions.read(),
                components.accelerations.write(),
            );

            entities
                .iter_mut()
                .find(|e| **e == event_entity)
                .and_then(|e| {
                    *e = e.kill();
                    Some(())
                })
                .expect("tried to kill non-existing entity");
        }

        MyWorld {
            components,
            entities,
            ..world
        }
    }
}
