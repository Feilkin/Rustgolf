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

use crate::systems::{
    physics::*,
    util::*,
};
use crate::world::MyWorld;

#[derive(Debug, Default)]
struct UiState {}

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

        let mut world = MyWorld {
            entities,
            components,
            ..world
        };

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
                SystemGroup::new(vec![
                    Box::new(MoveSystem::new()),
                    Box::new(CollisionGenerator::new()),
                    Box::new(CollisionResolver::new()),
                ]),
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
