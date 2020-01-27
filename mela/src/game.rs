//! here we go again

use glium::glutin::event::Event;
use glium::glutin::event_loop::ControlFlow;
use std::time::Duration;

pub trait Playable {
    fn update(self, delta: Duration, display: &glium::Display, ui: &mut imgui::Ui) -> Self;
    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow>;
    fn redraw(&self, display: &glium::Display, target: &mut glium::Frame);
}

#[derive(Default)]
pub struct IoState {
    pub mouse_position: [f32; 2],
    pub mouse_buttons: [bool; 3],
}
