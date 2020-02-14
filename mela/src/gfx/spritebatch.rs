//! Efficient batched sprites

use glium::uniform;
use nalgebra::Matrix4;

use crate::assets::Spritesheet;
use crate::gfx::{Quad, Texture, Vertex};

// TODO: fix typo
pub struct Spritebatch<'a> {
    spritesheet: &'a Spritesheet,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    pub quad_count: usize,
}

impl<'a> Spritebatch<'a> {
    pub fn new(spritesheet: &'a Spritesheet) -> Spritebatch<'a> {
        Spritebatch {
            vertices: Vec::new(),
            indices: Vec::new(),
            quad_count: 0,
            spritesheet,
        }
    }

    pub fn add_quad(self, quad_index: usize, position: [f32; 2]) -> Spritebatch<'a> {
        let Spritebatch {
            spritesheet,
            mut vertices,
            mut indices,
            quad_count,
        } = self;

        let (quad_vertices, quad_indices) = spritesheet
            .quad(quad_index)
            .vertices_and_indices(position, [16., 16.]);

        // offset indices
        let index_offset = vertices.len() as u16;

        vertices.extend_from_slice(&quad_vertices);
        indices.extend(quad_indices.iter().map(|i| i + index_offset));

        Spritebatch {
            quad_count: quad_count + 1,
            spritesheet,
            vertices,
            indices,
        }
    }

    pub fn draw(
        self,
        camera: Matrix4<f32>,
        display: &glium::Display,
        target: &mut glium::Frame,
        shader: &glium::Program,
    ) {
        use glium::Surface;

        let uniforms = uniform! {
            matrix: Into::<[[f32; 4]; 4]>::into(camera),
            tex: self.spritesheet.texture(),
        };

        let vertex_buffer = glium::VertexBuffer::new(display, &self.vertices).unwrap();
        let indices = glium::index::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &self.indices,
        )
        .unwrap();

        let mut draw_params: glium::draw_parameters::DrawParameters = Default::default();
        draw_params.blend = glium::Blend::alpha_blending();

        target
            .draw(&vertex_buffer, &indices, shader, &uniforms, &draw_params)
            .unwrap();
    }
}
