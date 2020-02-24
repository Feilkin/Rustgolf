//! Mesh stuff

use glium::texture::ResidentTexture;
use glium::uniform;
use nalgebra::Matrix4;

use crate::gfx::{Quad, Texture, Vertex};
use std::rc::Rc;

/// Mesh is a collection of vertices, indices, and a texture
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    texture: Rc<Texture>,
}

// constructors
impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>, texture: Rc<Texture>) -> Mesh {
        Mesh {
            vertices,
            indices,
            texture,
        }
    }
}

// getters
impl Mesh {
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

// drawing
impl Mesh {
    pub fn draw(
        &self,
        camera: &Matrix4<f32>,
        display: &glium::Display,
        target: &mut glium::Frame,
        shader: &glium::Program,
    ) {
        use glium::Surface;

        let uniforms = uniform! {
            matrix: Into::<[[f32; 4]; 4]>::into(camera.clone()),
            tex: self.texture(),
        };

        let vertex_buffer = glium::VertexBuffer::new(display, &self.vertices).unwrap();
        let indices = glium::index::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &self.indices,
        )
        .unwrap();

        target
            .draw(
                &vertex_buffer,
                &indices,
                shader,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}
