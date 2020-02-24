//! Tilemap objects

use crate::assets::tilemap::data;
use crate::debug::DebugDrawable;
use imgui::Ui;
use imgui_glium_renderer::Renderer;

pub struct Object {}

impl DebugDrawable for Object {
    fn draw_debug_ui(&mut self, ui: &Ui, _renderer: &mut Renderer) {}
}

impl From<data::Object> for Object {
    fn from(data: data::Object) -> Self {
        Object {}
    }
}

#[derive(Debug, Clone)]
pub struct ObjectGroup {}

impl DebugDrawable for ObjectGroup {
    fn draw_debug_ui(&mut self, ui: &Ui, _renderer: &mut Renderer) {}
}

impl From<&data::ObjectGroup> for ObjectGroup {
    fn from(data: &data::ObjectGroup) -> Self {
        ObjectGroup {}
    }
}
