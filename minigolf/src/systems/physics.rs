use crate::components::physics::{Acceleration, PhysicsEvent, Position, Velocity};
use mela::ecs::{ComponentStorage, Entity, ReadAccess, RwAccess, System, WriteAccess};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::world::MyWorld;
use mela::ecs::world::World;
use mela::nalgebra::Vector2;
use mela::profiler;
use mela::profiler::{OpenTagTreeRoot, PopTag, PushTag};

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

        let (screen_width, screen_height) = (1920. * 2., 1080. * 2.);

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
    fn name(&self) -> &'static str {
        "MoveSystem"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (MyWorld, profiler::OpenTagTree<'f>) {
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

        (
            MyWorld {
                last_physics_update: Instant::now(),
                entities,
                components,
                ..world
            },
            profiler_tag,
        )
    }
}

struct CollisionObjectData {
    entity: Entity,
}

type CollisionWorld = mela::ncollide2d::world::CollisionWorld<f32, CollisionObjectData>;

pub struct CollisionGenerator {
    world_handle_lookup: HashMap<Entity, mela::ncollide2d::pipeline::CollisionObjectSlabHandle>,
    collision_set: HashSet<(Entity, Entity)>,
    collision_world: CollisionWorld,
}

impl CollisionGenerator {
    pub fn new() -> CollisionGenerator {
        CollisionGenerator {
            world_handle_lookup: HashMap::new(),
            collision_set: HashSet::new(),
            collision_world: CollisionWorld::new(1.0),
        }
    }

    fn handle_contact_event(
        &self,
        event: &mela::ncollide2d::pipeline::ContactEvent<
            mela::ncollide2d::pipeline::CollisionObjectSlabHandle,
        >,
    ) -> PhysicsEvent {
        use mela::ncollide2d::pipeline::ContactEvent;

        match event {
            &ContactEvent::Started(cause_handle, other_handle) => {
                let cause_object = self.collision_world.collision_object(cause_handle).unwrap();
                let other_object = self.collision_world.collision_object(other_handle).unwrap();

                let (_, _, _, contacts) = self.collision_world.contact_pair(cause_handle, other_handle, true).unwrap();

                let deepest = contacts.deepest_contact().unwrap();

                PhysicsEvent::Collision {
                    cause: cause_object.data().entity,
                    other: other_object.data().entity,
                    contact: deepest.contact,
                    toi: 0.0
                }
            },
            &ContactEvent::Stopped(cause_handle, other_handle) => {
                let cause_object = self.collision_world.collision_object(cause_handle).unwrap();
                let other_object = self.collision_world.collision_object(other_handle).unwrap();

                PhysicsEvent::CollisionEnded {
                    cause: cause_object.data().entity,
                    other: other_object.data().entity,
                }
            },
        }
    }

    fn rebuild_collision_world(
        &mut self,
        positions: &mut dyn Iterator<Item = (Entity, &Position)>,
    ) {
        use mela::nalgebra::Isometry2;
        use mela::ncollide2d::pipeline::CollisionObjectSlabHandle;
        use mela::ncollide2d::shape::{Ball, ShapeHandle};

        let mut pending_update: HashSet<Entity> =
            self.world_handle_lookup.keys().cloned().collect();

        for (entity, position) in positions {
            match self.world_handle_lookup.get(&entity) {
                Some(handle) => {
                    let isometry = Isometry2::new(position.coords, 0.);

                    let object = self.collision_world.get_mut(*handle).unwrap();
                    object.set_position(isometry);
                }
                None => {
                    let shape_handle = ShapeHandle::new(Ball::new(8f32));
                    let isometry = Isometry2::new(position.coords, 0.);

                    let mut balls_group = mela::ncollide2d::pipeline::CollisionGroups::new();
                    balls_group.set_membership(&[1]);

                    let query_type =
                        mela::ncollide2d::pipeline::GeometricQueryType::Contacts(0., 0.);

                    let (handle, _) = self.collision_world.add(
                        isometry,
                        shape_handle,
                        balls_group,
                        query_type,
                        CollisionObjectData { entity },
                    );

                    self.world_handle_lookup.insert(entity, handle);
                }
            }

            pending_update.remove(&entity);
        }

        // entities left in pending_update have been removed from the world
        let to_remove: Vec<CollisionObjectSlabHandle> = pending_update
            .iter()
            .map(|e| *self.world_handle_lookup.get(e).unwrap())
            .collect();

        self.collision_world.remove(&to_remove);

        for entity in pending_update.iter() {
            self.world_handle_lookup.remove(entity);
        }
    }
}

impl System<MyWorld> for CollisionGenerator {
    fn name(&self) -> &'static str {
        "CollisionGenerator"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        mut profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (MyWorld, profiler::OpenTagTree<'f>) {
        let MyWorld {
            entities,
            mut components,
            ..
        } = world;

        components.physics_events.write().clear();

        let mut new_events = Vec::new();

        let rebuilder_tag =
            profiler_tag.push_tag("rebuilding collision world", [0.3, 0.5, 0.2, 1.0]);
        self.rebuild_collision_world(&mut components.positions.read().iter());

        let mut world_update_tag = rebuilder_tag.pop_tag().push_tag("updating world", [0.3, 0.2, 0.5, 1.0]);
        {
            self.collision_world.clear_events();

            let broad_phase_tag = world_update_tag.push_tag("broad phase", [0.8, 0.4, 0.4, 1.0]);
            self.collision_world.perform_broad_phase();
            let narrow_phase_tag = broad_phase_tag.pop_tag().push_tag("narrow phase", [0.4, 0.8, 0.4, 1.0]);
            self.collision_world.perform_narrow_phase();

            world_update_tag = narrow_phase_tag.pop_tag();
        }



        let mut generator_tag = world_update_tag
            .pop_tag()
            .push_tag("generating collision events", [0.8, 0.8, 0.2, 1.0]);

        for event in self.collision_world.contact_events() {
            new_events.push(self.handle_contact_event(event));
        }

        let mut new_world = MyWorld {
            entities,
            components,
            ..world
        };

        let pushing_tag = generator_tag
            .pop_tag()
            .push_tag("pushing events", [0.2, 0.2, 0.9, 1.0]);

        for event in new_events {
            new_world = new_world.add_entity().with_component(event).build()
        }

        profiler_tag = pushing_tag.pop_tag();

        (new_world, profiler_tag)
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
                    (pos_diff.normalize() * (contact.depth * 10.)).into(),
                );
                accelerations.set(
                    other,
                    (pos_diff2.normalize() * (contact.depth * 10.)).into(),
                );
            }
            _ => (),
        }
    }
}

impl System<MyWorld> for CollisionResolver {
    fn name(&self) -> &'static str {
        "CollisionResolver"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (MyWorld, profiler::OpenTagTree<'f>) {
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

        (
            MyWorld {
                components,
                entities,
                ..world
            },
            profiler_tag,
        )
    }
}
