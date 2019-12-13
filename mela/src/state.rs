//! Stateful game states manager factory

pub trait State {
    fn name(&self) -> &str;
    fn load(&mut self, display: &glium::Display);
    fn focus(&mut self, display: &glium::Display);
    fn leave(&mut self, display: &glium::Display);
    fn update(&mut self, display: &glium::Display);
    fn redraw(&self, display: &glium::Display, target: &mut glium::Frame);
}