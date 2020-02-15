//! Debugging utilities

use imgui::Ui;
use imgui_glium_renderer::Renderer;

pub trait DebugDrawable {
    fn draw_debug_ui(&mut self, ui: &Ui, renderer: &mut Renderer) {}
}
