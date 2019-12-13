//! spröööt :-DD

use mela::glium::Display;
use std::cell::RefCell;
use std::rc::Rc;

use mela::state::State;

pub mod loading_screen;

pub use loading_screen::LoadingScreen;

pub fn states() -> Vec<Rc<RefCell<dyn State>>> {
    vec![Rc::new(RefCell::new(LoadingScreen::new()))]
}
