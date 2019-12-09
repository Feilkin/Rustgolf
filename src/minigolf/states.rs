//! spröööt :-DD

use glium::{uniform, Surface};

use crate::game::gfx;

use crate::game::assets::image::Image;
use crate::game::state::State;
use glium::uniforms::UniformsStorage;

pub struct LoadingScreen {
    assets: Vec<Image>,
    img_shader: glium::Program,
}

impl LoadingScreen {
    pub fn new(display: &glium::Display) -> LoadingScreen {
        let vertex_shader_src = include_str!("../shaders/simple_texture.vertex.glsl");
        let fragment_shader_src = include_str!("../shaders/simple_texture.fragment.glsl");
        let img_shader = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        LoadingScreen {
            assets: Vec::new(),
            img_shader
        }
    }
}

impl State for LoadingScreen {
    fn focus(&mut self, display: &glium::Display) {
        self.assets.push(Image::from_file("assets/loading.png", display).unwrap());
    }

    fn leave(&mut self, display: &glium::Display) {
    }

    fn update(&mut self, display: &glium::Display) {
    }

    fn redraw(&self, display: &glium::Display, target: &mut glium::Frame) {
        println!("   - redrawing state");

        let (width, height) = (800.0, 600.0);

        let camera_matrix = nalgebra::Matrix4::new_orthographic(0.0_f32, width, height, 0.0, 0.0, 10.0)
            .append_scaling(0.5);

        for img in &self.assets {
            println!("     - drawing img");
            let uniforms = uniform!{
                matrix: Into::<[[f32; 4]; 4]>::into(camera_matrix),
                tex: img.texture(),
            };

            use crate::game::gfx::Vertex;

            let (iw, ih) = (img.width() as f32, img.height() as f32);

            let shape = vec![
                Vertex { position: [0.0, 0.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [0.0, ih], tex_coords: [0.0, 1.0] },
                Vertex { position: [iw, ih], tex_coords: [1.0, 1.0] },
                Vertex { position: [iw, 0.0], tex_coords: [1.0, 0.0] },
            ];

            let indices = vec![0u16, 1, 2, 0, 2, 3];

            let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
            let indices = glium::index::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

            target.draw(&vertex_buffer, &indices, &self.img_shader, &uniforms, &Default::default()).unwrap();
        }
    }
}