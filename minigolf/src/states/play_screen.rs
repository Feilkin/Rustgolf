use crate::components::physics::{Acceleration, Velocity};
use crate::components::{physics::Position, GolfComponents};
use crate::states::{LoadingScreen, State as GolfState};
use imgui_glium_renderer::imgui::Ui;
use itertools::izip;
use mela::ecs::{Component, ComponentStorage, Entity, ReadAccess, World, WriteAccess};
use mela::game::IoState;
use mela::gfx::Spritebatch;
use mela::glium::{Display, Frame, Program};
use mela::state::State;
use mela::{glium, nalgebra};
use std::fmt::{Debug, Error, Formatter};
use std::net::Shutdown::Write;
use std::time::Duration;

#[derive(Debug, Default)]
struct UiState {
    demo_window_open: bool,
}

pub struct PlayScreen {
    img_shader: Program,
    spritesheet: mela::assets::Spritesheet,
    ui_state: UiState,
    world: mela::ecs::World<GolfComponents>,
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
        _io_state: &IoState,
    ) -> GolfState {
        let mut demo_window_open = self.ui_state.demo_window_open;
        ui.show_demo_window(&mut demo_window_open);

        // TODO: ECS stuff
        let World {
            entities,
            mut components,
            ..
        } = self.world;

        for entity in &entities {
            match (
                components.fetch(*entity),
                components.fetch(*entity),
                components.fetch(*entity),
            ) {
                (Some(p), Some(v), Some(a)) => {
                    let (position, velocity) = move_entity(delta, p, v, a);
                    components.write(*entity, position);
                    components.write(*entity, velocity);
                }
                _ => (),
            }
        }

        // DEBUGGING
        use mela::imgui::im_str;
        ui.text(im_str!("entities: {}", entities.len()));
        ui.text(im_str!("fps: {}", 1.0 / delta.as_secs_f64()));

        GolfState::Play(PlayScreen {
            ui_state: UiState {
                demo_window_open,
                ..self.ui_state
            },
            world: World {
                entities,
                components,
                ..self.world
            },
            ..self
        })
    }

    fn redraw(&self, display: &Display, target: &mut Frame) {
        let (width, height) = (800.0, 600.0);

        let camera_matrix =
            nalgebra::Matrix4::new_orthographic(0.0_f32, width, height, 0.0, 0.0, 10.0);

        let mut spritebatch = Spritebatch::new(&self.spritesheet);

        for maybe_position in self.world.components.fetch_all::<Position>() {
            match maybe_position {
                Some(position) => spritebatch = spritebatch.add_quad(0, [position.x, position.y]),
                None => (),
            }
        }

        spritebatch.draw(camera_matrix, display, target, &self.img_shader);
    }
}

impl From<LoadingScreen> for PlayScreen {
    fn from(l: LoadingScreen) -> Self {
        let (img_shader, spritesheet) = l.assets();

        let mut world = World::new(GolfComponents::new());

        for x in 0..40 {
            for y in 0..30 {
                world = world
                    .add_entity()
                    .with_component(Position::new(20. * x as f32, 20. * y as f32))
                    .with_component(Velocity::new(0., 0.))
                    .with_component(Acceleration::new(10., 0.))
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
        position.x = -16.
    }

    (position.into(), (velocity + half_of_velocity_delta).into())
}
