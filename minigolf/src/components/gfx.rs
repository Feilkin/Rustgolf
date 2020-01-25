//! graphics components

use std::sync::Arc;

use mela::gfx::{Quad, TextureId, Vertex};
use mela::nalgebra::{Point2, Vector2};

/// 2D sprite (texture + quad)
#[derive(Debug)]
pub struct Sprite {
    texture: TextureId,
    quad: Quad,
    origin: Point2<f32>,
}

// Constructors
impl Sprite {
    pub fn new(texture: TextureId, quad: Quad) -> Sprite {
        Sprite {
            origin: Point2::new(0.0, 0.0),
            texture,
            quad,
        }
    }

    pub fn origin(self, origin: Point2<f32>) -> Sprite {
        Sprite { origin, ..self }
    }
}

// getters
impl Sprite {
    pub fn texture(&self) -> TextureId {
        self.texture
    }

    pub fn vertices_and_indices(&self) -> ([Vertex; 4], [u16; 6]) {
        self.quad
            .vertices_and_indices([-self.origin.x, -self.origin.y], [1.0, 1.0])
    }
}
