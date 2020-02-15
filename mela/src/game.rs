//! here we go again

use std::time::Duration;

use glium::glutin::event::Event;
use glium::glutin::event_loop::ControlFlow;

use crate::profiler;
use imgui_glium_renderer::Renderer;

pub trait Playable: Sized {
    fn update(
        self,
        delta: Duration,
        display: &glium::Display,
        ui: &mut imgui::Ui,
        renderer: &mut Renderer,
        profiler_frame: &mut profiler::OpenFrame,
    ) -> Self;
    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow>;
    fn redraw(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        profiler_frame: &mut profiler::OpenFrame,
    );
}

#[derive(Default)]
pub struct IoState {
    pub mouse_position: [f32; 2],
    pub mouse_buttons: [bool; 3],
}
