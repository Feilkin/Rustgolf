//! here we go again

use glutin::event::Event;
use glutin::event_loop::ControlFlow;

pub(crate) mod assets;
pub mod gfx;
pub(crate) mod state;

pub trait Playable {
    fn update(&mut self, display: &glium::Display);
    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow>;
    fn redraw(&self, display: &glium::Display, target: &mut glium::Frame);
}