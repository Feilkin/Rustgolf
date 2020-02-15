//! Actual tilemap implementation lives here

use crate::assets::tilemap::{Tileset, data};
use crate::debug::DebugDrawable;
use imgui_glium_renderer::Renderer;
use imgui::Ui;
use glium::Display;
use std::path::Path;
use std::fs::File;
use crate::assets::{AssetError};
use std::io::BufReader;
use itertools::Itertools;

pub trait Orientation {
    fn from_data(data: &data::Map) -> Self;
}

pub struct Tilemap<O: Orientation> {
    name: String,
    size: (usize, usize),
    tile_size: (usize, usize),
    tilesets: Vec<Tileset>,
    orientation: O,
}

impl<O: Orientation> Tilemap<O> {
    pub fn from_file<P: AsRef<Path>>(path: P, display: &Display) -> Result<Tilemap<O>, AssetError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        let name = path
            .as_ref()
            .file_name()
            .and_then(|os_str| Some(os_str.to_string_lossy()))
            .unwrap_or("unnamed".into())
            .into_owned();

        Tilemap::from_data(data, name, path, display)
    }

    pub fn from_data<P: AsRef<Path>>(data: data::Map, name: String, path: P, display: &Display) -> Result<Tilemap<O>, AssetError> {
        let orientation = O::from_data(&data);
        let tilesets = data.tilesets
            .into_iter()
            .map(|inlined_or_external|
                     inlined_or_external
                         .into_tileset(path.as_ref())
                         .and_then(|data|
                             Tileset::build(data, path.as_ref(), display)))
            .collect::<Result<Vec<Tileset>, AssetError>>()?;

        Ok(Tilemap {
            name,
            orientation,
            tilesets,
            size: (data.width, data.height),
            tile_size: (data.tilewidth, data.tileheight),
        })
    }
}

// Tilemap orientations


pub struct Orthogonal {
    render_order: data::RenderOrder,
}

impl Orientation for Orthogonal {
    fn from_data(data: &data::Map) -> Self {
        if data.orientation != data::MapOrientation::Orthogonal {
            panic!("Attempted to use non-orthogonal map as orthogonal!");
        }

        Orthogonal {
            render_order: data.renderorder.as_ref().unwrap().clone(),
        }
    }
}

impl DebugDrawable for Orthogonal {
    fn draw_debug_ui(&mut self, ui: &Ui, renderer: &mut Renderer) {
        use imgui::*;

        ui.text(im_str!("Orientation: orthogonal"));
        ui.text(&im_str!("Render order: {:?}", self.render_order));
    }
}

impl<O: Orientation + DebugDrawable> DebugDrawable for Tilemap<O> {
    fn draw_debug_ui(&mut self, ui: &Ui, renderer: &mut Renderer) {
        use imgui::*;

        ui.text(&im_str!("width:       {}", self.size.0));
        ui.text(&im_str!("height:      {}", self.size.1));
        ui.text(&im_str!("tile width:  {}", self.tile_size.0));
        ui.text(&im_str!("tile height: {}", self.tile_size.1));

        self.orientation.draw_debug_ui(ui, renderer);

        ui.tree_node(&im_str!("tilemap-{}-tilesets", self.name))
            .label(im_str!("Tilesets"))
            .build(|| {
                for (index, tileset) in self.tilesets.iter_mut().enumerate() {
                    ui.tree_node(&im_str!("tilemap-{}-tilesets-{}", self.name, index))
                        .label(&im_str!("Tileset {}", index))
                        .build(|| tileset.draw_debug_ui(ui, renderer));
                }
            });
    }
}