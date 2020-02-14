//! Graphics related stuff

use glium::implement_vertex;
pub use glium::texture::Texture2d as Texture;

pub use mesh::Mesh;
pub use spritebatch::Spritebatch;

use crate::assets::image::Image;

mod mesh;
mod spritebatch;

#[derive(Debug, Clone, Copy)]
pub struct TextureId(usize);

impl From<usize> for TextureId {
    fn from(id: usize) -> Self {
        TextureId(id)
    }
}

impl From<TextureId> for usize {
    fn from(id: TextureId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    position: [f32; 2],
    size: [f32; 2],
}

impl Quad {
    pub fn new(position: [f32; 2], size: [f32; 2], source_dimensions: (u32, u32)) -> Quad {
        let sw = source_dimensions.0 as f32;
        let sh = source_dimensions.1 as f32;

        Quad {
            position: [position[0] / sw, position[1] / sh],
            size: [size[0] / sw, size[1] / sh],
        }
    }

    pub fn vertices_and_indices(
        &self,
        position: [f32; 2],
        size: [f32; 2],
    ) -> ([Vertex; 4], [u16; 6]) {
        let (vx0, vy0) = (position[0], position[1]);
        let (vx1, vy1) = (vx0 + size[0], vy0 + size[1]);
        let (tx0, ty0) = (self.position[0], self.position[1]);
        let (tx1, ty1) = (tx0 + self.size[0], ty0 + self.size[1]);

        let vertices = [
            Vertex {
                position: [vx0, vy0],
                tex_coords: [tx0, ty0],
            },
            Vertex {
                position: [vx0, vy1],
                tex_coords: [tx0, ty1],
            },
            Vertex {
                position: [vx1, vy1],
                tex_coords: [tx1, ty1],
            },
            Vertex {
                position: [vx1, vy0],
                tex_coords: [tx1, ty0],
            },
        ];

        let indices = [0, 1, 2, 0, 2, 3];

        (vertices, indices)
    }
}

implement_vertex!(Vertex, position, tex_coords);
