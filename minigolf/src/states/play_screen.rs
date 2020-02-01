use crate::components::physics::{Acceleration, PhysicsEvent, Velocity};
use crate::components::{physics::Position, GolfComponents};
use crate::states::{LoadingScreen, State as GolfState, StateDebugger};
use imgui_glium_renderer::imgui::Ui;
use itertools::izip;
use mela::ecs::entity::EntityBuilder;
use mela::ecs::world::{World, WorldStorage};
use mela::ecs::{
    Component, ComponentStorage, DequeStorage, Entity, ReadAccess, System, VecReader, VecStorage,
    VecWriter, WriteAccess,
};
use mela::game::IoState;
use mela::gfx::Spritebatch;
use mela::glium::{Display, Frame, Program};
use mela::state::State;
use mela::{glium, nalgebra};
use nalgebra::{Point2, Vector2};
use std::collections::HashSet;
use std::fmt::{Debug, Error, Formatter};
use std::net::Shutdown::Write;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
struct UiState {}

struct MyWorld {
    next_entity_id: usize,
    entities: Vec<Entity>,
    components: GolfComponents,
    last_physics_update: Instant,
}

impl MyWorld {
    fn new() -> MyWorld {
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

        let new_entity = Entity(next_entity_id);
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

pub struct PlayScreen {
    img_shader: Program,
    spritesheet: mela::assets::Spritesheet,
    ui_state: UiState,
    world: MyWorld,
    systems: Vec<Box<dyn System<MyWorld>>>,
    last_frame_delta: Duration,
    is_debugged: bool,
}

impl Debug for PlayScreen {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "PlayScreen")?;
        Ok(())
    }
}

impl State for PlayScreen {
    type Wrapper = GolfState;

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn load(&mut self, display: &Display) {
        unimplemented!()
    }

    fn focus(&mut self, display: &Display) {
        unimplemented!()
    }

    fn leave(&mut self, display: &Display) {
        unimplemented!()
    }

    fn update(
        self,
        delta: Duration,
        _display: &Display,
        ui: &mut mela::imgui::Ui,
        io_state: &IoState,
    ) -> GolfState {
        use mela::imgui::*;
        if !self.is_debugged {
            if ui.button(im_str!("debug!"), [0., 0.]) {
                return GolfState::StateDebugger(Box::new(StateDebugger::new(GolfState::Play(
                    PlayScreen {
                        is_debugged: true,
                        ..self
                    },
                ))));
            }
        }

        // TODO: ECS stuff
        let mut systems = self.systems;
        let mut world = self.world;

        for system in systems.iter_mut() {
            world = system.update(delta, world);
        }

        let MyWorld {
            entities,
            mut components,
            ..
        } = world;

        // player input
        match entities.first() {
            Some(entity) => {
                let maybe_velocity = player_input(
                    delta,
                    components.positions.read().fetch(*entity).unwrap(),
                    &io_state,
                );

                match maybe_velocity {
                    Some(velocity) => components.velocities.write().set(*entity, velocity),
                    None => (),
                }
            }
            None => (), // no balls, no interaction
        }

        // move

        // collide
        let mut collision_set = HashSet::new();
        let mut new_events = Vec::new();
        for entity in &entities {
            match (
                components.positions.read().fetch(*entity),
                components.velocities.read().fetch(*entity),
            ) {
                (Some(p), Some(v)) => {
                    let collisions = collide_entities(
                        &mut collision_set,
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

        // handle collisions (these are frame late due to reasons)
        let mut new_velocities = Vec::new();
        for (event_entity, event) in components.physics_events.read().iter() {
            match resolve_collision(
                event,
                components.velocities.read(),
                components.positions.read(),
            ) {
                Some(stuff) => {
                    new_velocities.push((event_entity, stuff));
                }
                None => (),
            }
        }

        // clear events
        components.physics_events.write().clear();

        let mut world = MyWorld {
            entities,
            components,
            ..world
        };

        for (event_entity, (e1, v1, e2, v2)) in new_velocities {
            world.components.velocities.write().set(e1, v1);
            world.components.velocities.write().set(e2, v2);

            // remove entity???
        }

        for event in new_events {
            world = world.add_entity().with_component(event).build()
        }

        GolfState::Play(PlayScreen {
            ui_state: UiState { ..self.ui_state },
            last_frame_delta: delta,
            systems,
            world,
            ..self
        })
    }

    fn redraw(&self, display: &Display, target: &mut Frame) {
        let (width, height) = (800.0, 600.0);

        let camera_matrix =
            nalgebra::Matrix4::new_orthographic(0.0_f32, width, height, 0.0, 0.0, 10.0);

        let mut spritebatch = Spritebatch::new(&self.spritesheet);

        let world = &self.world;
        for entity in &world.entities {
            match (
                world.components.positions.read().fetch(*entity),
                world.components.velocities.read().fetch(*entity),
            ) {
                (Some(p), Some(v)) => {
                    let (p, v) = (**p, **v);

                    // dirty hack to disable extrapolation when debugging lol
                    let pos = if !self.is_debugged {
                        let since_last_physics_update = Instant::now() - world.last_physics_update;
                        &p + &v * since_last_physics_update.as_secs_f32()
                    } else {
                        p
                    };

                    spritebatch =
                        spritebatch.add_quad(0, [pos.x, pos.y]);
                }
                _ => (),
            }
        }

        spritebatch.draw(camera_matrix, display, target, &self.img_shader);
    }

    fn update_debug_ui(&mut self, ui: &mut mela::imgui::Ui) {
        use mela::imgui::*;

        Window::new(im_str!("Entities"))
            .size([400., 300.], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("total count: {}", self.world.entities.len()));

                for entity in &self.world.entities {
                    if CollapsingHeader::new(ui, &im_str!("Entity #{}", **entity))
                        .bullet(true)
                        .build() {
                        if let Some(p) = self.world.components.positions.read().fetch(*entity) {
                            let mut value: [f32; 2] = p.coords.into();

                            ui.input_float2(&im_str!("Position"), &mut value)
                                .build();
                        }
                        if let Some(p) = self.world.components.velocities.read().fetch(*entity) {
                            let mut value: [f32; 2] = (**p).into();

                            ui.input_float2(&im_str!("Velocity"), &mut value)
                                .build();
                        }
                        if let Some(p) = self.world.components.accelerations.read().fetch(*entity) {
                            let mut value: [f32; 2] = (**p).into();

                            ui.input_float2(&im_str!("Acceleration"), &mut value)
                                .build();
                        }
                    }
                }
            });
    }
}

impl From<LoadingScreen> for PlayScreen {
    fn from(l: LoadingScreen) -> Self {
        let (img_shader, spritesheet) = l.assets();

        let mut world = MyWorld::new();

        for x in 0..10 {
            for y in 0..7 {
                world = world
                    .add_entity()
                    .with_component(Position::new(80. * x as f32, 80. * y as f32))
                    .with_component(Velocity::new(0., 0.))
                    .with_component(Acceleration::new(0., 0.))
                    .build();
            }
        }

        println!("done setting up world");

        PlayScreen {
            ui_state: UiState::default(),
            systems: vec![Box::new(FixedInterval::wrap(
                MoveSystem::new(),
                Duration::from_millis(20),
            ))],
            last_frame_delta: Duration::new(0, 0),
            is_debugged: false,
            world,
            img_shader,
            spritesheet,
        }
    }
}

fn player_input(delta: Duration, position: &Position, io: &IoState) -> Option<Velocity> {
    if io.mouse_buttons[0] {
        // calculate difference between ball and click
        let [mx, my] = io.mouse_position;
        let delta: Vector2<f32> = Point2::new(mx, my) - &position.0;

        Some(Velocity::new_from(delta))
    } else {
        None
    }
}

struct FixedInterval<S: System<MyWorld>> {
    inner: S,
    delta_buffer: Duration,
    tick_interval: Duration,
}

impl<S: System<MyWorld>> FixedInterval<S> {
    pub fn wrap(system: S, interval: Duration) -> FixedInterval<S> {
        FixedInterval {
            inner: system,
            tick_interval: interval,
            delta_buffer: Duration::new(0, 0),
        }
    }
}

impl<S: System<MyWorld>> System<MyWorld> for FixedInterval<S> {
    fn update(&mut self, delta: Duration, world: MyWorld) -> MyWorld {
        self.delta_buffer += delta;
        let mut world = world;

        while self.delta_buffer >= self.tick_interval {
            world = self.inner.update(self.tick_interval, world);

            self.delta_buffer -= self.tick_interval;
        }

        world
    }
}

struct MoveSystem {}

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

        if position.x > 808. {
            position.x = -8.
        } else if position.x < -8. {
            position.x = 808.
        }

        if position.y > 608. {
            position.y = -8.
        } else if position.y < -8. {
            position.y = 608.
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

fn collide_entities(
    collision_set: &mut HashSet<(Entity, Entity)>,
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
        if collision_set.contains(&(other_entity, *entity)) {
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

                    collision_set.insert((*entity, other_entity));
                }
            }
            _ => (), // no collision
        }
    }

    events
}

fn resolve_collision<'r, V: ReadAccess<'r, Velocity>, P: ReadAccess<'r, Position>>(
    collision: &PhysicsEvent,
    velocities: V,
    positions: P,
) -> Option<(Entity, Velocity, Entity, Velocity)> {
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
            let cause_position = contact.world1;
            let other_position = contact.world2;

            let pos_diff = &cause_position - &other_position;
            let pos_diff2 = &other_position - &cause_position;

            // TODO: implement mass
            let (cause_mass, other_mass) = (1f32, 1f32);

            let new_cause_velocity = &cause_velocity
                - ((2. * other_mass) / (cause_mass + other_mass))
                    * ((&cause_velocity - &other_velocity).dot(&pos_diff)
                        / (pos_diff.norm().powf(2.0)))
                    * pos_diff;

            let new_other_velocity = &other_velocity
                - ((2. * cause_mass) / (cause_mass + other_mass))
                    * ((&other_velocity - &cause_velocity).dot(&pos_diff2)
                        / (pos_diff2.norm().powf(2.0)))
                    * pos_diff2;

            Some((
                cause,
                new_cause_velocity.into(),
                other,
                new_other_velocity.into(),
            ))
        }
        _ => None,
    }
}
