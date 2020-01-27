use crate::components::physics::{Acceleration, PhysicsEvent, Velocity};
use crate::components::{physics::Position, GolfComponents};
use crate::states::{LoadingScreen, State as GolfState};
use imgui_glium_renderer::imgui::Ui;
use itertools::izip;
use mela::ecs::entity::EntityBuilder;
use mela::ecs::world::{World, WorldStorage};
use mela::ecs::{Component, ComponentStorage, Entity, ReadAccess, VecWriter, WriteAccess, VecStorage, VecReader, DequeStorage};
use mela::game::IoState;
use mela::gfx::Spritebatch;
use mela::glium::{Display, Frame, Program};
use mela::state::State;
use mela::{glium, nalgebra};
use nalgebra::{Point2, Vector2};
use std::fmt::{Debug, Error, Formatter};
use std::net::Shutdown::Write;
use std::time::Duration;

#[derive(Debug, Default)]
struct UiState {
    demo_window_open: bool,
}

struct MyWorld {
    next_entity_id: usize,
    entities: Vec<Entity>,
    components: GolfComponents,
}

impl MyWorld {
    fn new() -> MyWorld {
        MyWorld {
            next_entity_id: 0,
            entities: Vec::new(),
            components: GolfComponents::default(),
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
        let mut demo_window_open = self.ui_state.demo_window_open;
        ui.show_demo_window(&mut demo_window_open);

        // TODO: ECS stuff
        let MyWorld {
            entities,
            mut components,
            ..
        } = self.world;

        // DEBUGGING
        use mela::imgui::im_str;
        ui.text(im_str!("entities: {}", entities.len()));
        ui.text(im_str!("fps: {}", 1.0 / delta.as_secs_f64()));

        // clear events
        //WriteAccess::<PhysicsEvent>::clear(&mut components);

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
        for entity in &entities {
            match (
                components.positions.read().fetch(*entity),
                components.velocities.read().fetch(*entity),
                components.accelerations.read().fetch(*entity),
            ) {
                (Some(p), Some(v), Some(a)) => {
                    let (position, velocity) = move_entity(delta, p, v, a);
                    components.positions.write().set(*entity, position);
                    components.velocities.write().set(*entity, velocity);
                }
                _ => (),
            }
        }

        // collide
        let mut new_events = Vec::new();
        for entity in &entities {
            match (
                components.positions.read().fetch(*entity),
                components.velocities.read().fetch(*entity),
            ) {
                (Some(p), Some(v)) => {
                    let collisions =
                        collide_entities(delta, entity, p, v, &mut components.positions.read().iter());
                    new_events.extend(collisions);
                }
                _ => (),
            }
        }

        let mut world = MyWorld {
            entities,
            components,
            ..self.world
        };

        for event in new_events {
            world = world
                .add_entity()
                .with_component(event)
                .build()
        }

        GolfState::Play(PlayScreen {
            ui_state: UiState {
                demo_window_open,
                ..self.ui_state
            },
            world,
            ..self
        })
    }

    fn redraw(&self, display: &Display, target: &mut Frame) {
        let (width, height) = (800.0, 600.0);

        let camera_matrix =
            nalgebra::Matrix4::new_orthographic(0.0_f32, width, height, 0.0, 0.0, 10.0);

        let mut spritebatch = Spritebatch::new(&self.spritesheet);

        for (_entity, position) in self.world.components.positions.read().iter() {
            spritebatch = spritebatch.add_quad(0, [position.x, position.y]);
        }

        spritebatch.draw(camera_matrix, display, target, &self.img_shader);
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
    }

    (
        position.into(),
        (velocity * 0.998 + half_of_velocity_delta).into(),
    )
}

fn collide_entities(
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
                    })
                }
            }
            _ => (), // no collision
        }
    }

    events
}
