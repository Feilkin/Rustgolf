//! Tilemap layers

use crate::assets::tilemap::tile::Tile;
use crate::gfx::{Mesh, Spritebatch};
use glium::{Display, Frame, Program};
use nalgebra::Matrix4;
use std::rc::Rc;

pub trait Layer {
    fn draw(&self, camera: &Matrix4<f32>, display: &Display, target: &mut Frame, shader: &Program);
}

pub struct TileLayer {
    data: Vec<Option<Tile>>,
    id: usize,
    name: String,
    offset: [f32; 2],
    size: (usize, usize),
}

impl TileLayer {
    pub fn new(
        data: Vec<Option<Tile>>,
        id: usize,
        name: String,
        offset: [f32; 2],
        size: (usize, usize),
    ) -> TileLayer {
        TileLayer {
            data,
            id,
            name,
            offset,
            size
        }
    }
}

impl Layer for TileLayer {
    fn draw(&self, camera: &Matrix4<f32>, display: &Display, target: &mut Frame, shader: &Program) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for row in 0..self.size.1 {
            for column in 0..self.size.0 {
                let id = column + row * self.size.0;

                // TODO: tilesize needs to come from parent, not from tile
                let tile_size = [16., 16.];

                if let Some(tile) = &self.data[id] {
                    let position = [
                        self.offset[0] + column as f32 * tile_size[0],
                        self.offset[1] + row as f32 * tile_size[1],
                    ];

                    let index_offset = vertices.len() as u16;

                    let (tile_vertices, tile_indices) = tile.quad().vertices_and_indices(position, tile_size);
                    vertices.extend_from_slice(&tile_vertices);
                    indices.extend(tile_indices.iter().map(|i| i + index_offset));
                }
            }
        }

        Mesh::new(
            vertices,
            indices,
            Rc::clone(self.data[0].as_ref().unwrap().image().texture()),
        )
            .draw(camera, display, target, shader);
    }
}
