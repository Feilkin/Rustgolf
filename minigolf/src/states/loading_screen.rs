//! State for loading :)

use std::fmt::{Debug, Error, Formatter};
use std::time::Duration;

use glium::uniforms::UniformsStorage;
use glium::{uniform, Display, Surface};

use mela::assets::{Image, Spritesheet};
use mela::game::IoState;
use mela::gfx;
use mela::gfx::{Mesh, Quad};
use mela::profiler::Profiler;
use mela::state::State;
use mela::{glium, nalgebra, profiler};

use crate::states::PlayScreen;
use crate::states::State as GolfState;
use mela::debug::DebugDrawable;
use mela::assets::tilemap::Tileset;

pub struct LoadingScreen {
    loading_img: Option<Mesh>,
    img_shader: Option<glium::Program>,
    spritesheet: Option<Spritesheet>,
    tileset: Option<Tileset>,
}

impl Debug for LoadingScreen {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "LoadingScreen")?;
        Ok(())
    }
}

impl LoadingScreen {
    pub fn new() -> LoadingScreen {
        LoadingScreen {
            loading_img: None,
            img_shader: None,
            spritesheet: None,
            tileset: None,
        }
    }

    pub fn done_loading(&self) -> bool {
        self.spritesheet.is_some() && self.tileset.is_some()
    }

    pub fn assets(self) -> (glium::Program, Spritesheet, Tileset) {
        (self.img_shader.unwrap(), self.spritesheet.unwrap(), self.tileset.unwrap())
    }
}

impl State for LoadingScreen {
    type Wrapper = GolfState;

    fn name(&self) -> &str {
        "loading_screen"
    }

    fn load(&mut self, display: &Display) {
        let vertex_shader_src = include_str!("../../src/shaders/simple_texture.vertex.glsl");
        let fragment_shader_src = include_str!("../../src/shaders/simple_texture.fragment.glsl");
        let img_shader =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        self.img_shader = Some(img_shader);

        let img = Image::from_file("assets/loading.png", display).unwrap();
        let quad = Quad::new([0.0, 0.0], [800., 600.], img.dimensions());
        let (vertices, indices) = quad.vertices_and_indices([0.0, 0.0], [800., 600.]);
        self.loading_img = Some(Mesh::new(vertices.to_vec(), indices.to_vec(), img.into()));
    }

    fn focus(&mut self, display: &glium::Display) {}

    fn leave(&mut self, display: &glium::Display) {}

    fn update(
        self,
        _delta: Duration,
        display: &glium::Display,
        ui: &mut mela::imgui::Ui,
        _io_state: &IoState,
        _profiler_frame: &mut profiler::OpenFrame,
    ) -> GolfState {
        if self.done_loading() {
            GolfState::Play(PlayScreen::from(self))
        } else {
            // TODO: async loading
            let spritesheet =
                Spritesheet::from_file("assets/sprites/balls/basic.json", display).unwrap();

            let tileset = Tileset::from_file("assets/maps/basic.tsx", display).unwrap();

            GolfState::Loading(LoadingScreen {
                spritesheet: Some(spritesheet),
                tileset: Some(tileset),
                ..self
            })
        }
    }

    fn redraw(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        _profiler_frame: &mut profiler::OpenFrame,
    ) {
        let (width, height) = (800.0, 600.0);

        let camera_matrix =
            nalgebra::Matrix4::new_orthographic(0.0_f32, width, height, 0.0, 0.0, 10.0);

        self.loading_img.as_ref().unwrap().draw(
            camera_matrix,
            display,
            target,
            self.img_shader.as_ref().unwrap(),
        );
    }
}

impl DebugDrawable for LoadingScreen {}