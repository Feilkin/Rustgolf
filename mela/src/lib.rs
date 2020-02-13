//! My game framework
#![feature(generic_associated_types)]

pub mod assets;
pub mod ecs;
pub mod game;
pub mod gfx;
pub mod profiler;
pub mod state;

pub use glium;
pub use glium::glutin;
pub use imgui;
pub use nalgebra;
pub use ncollide2d;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
