//! My game framework

pub mod assets;
pub mod game;
pub mod gfx;
pub mod state;

pub use glium;
pub use glutin;
pub use nalgebra;
pub use ncollide2d;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
