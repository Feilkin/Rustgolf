//! spröööt :-DD

use mela::glium::{Display, Frame};
use std::cell::RefCell;
use std::rc::Rc;

use mela::state::State as MelaState;

mod loading_screen;
mod play_screen;

pub use loading_screen::LoadingScreen;
use mela::game::IoState;
pub use play_screen::PlayScreen;
use std::ops::Deref;
use std::time::Duration;

#[derive(Debug)]
pub enum State {
    Loading(LoadingScreen),
    Play(PlayScreen),
}

impl MelaState for State {
    type Wrapper = State;

    fn name(&self) -> &str {
        match self {
            State::Loading(s) => s.name(),
            State::Play(s) => s.name(),
        }
    }

    fn load(&mut self, display: &Display) {
        match self {
            State::Loading(s) => s.load(display),
            State::Play(s) => s.load(display),
        }
    }

    fn focus(&mut self, display: &Display) {
        unimplemented!()
    }

    fn leave(&mut self, display: &Display) {
        unimplemented!()
    }

    fn update(
        self,
        delta: Duration,
        display: &Display,
        ui: &mut mela::imgui::Ui,
        io_state: &IoState,
    ) -> State {
        match self {
            State::Loading(s) => s.update(delta, display, ui, io_state),
            State::Play(s) => s.update(delta, display, ui, io_state),
        }
    }

    fn redraw(&self, display: &Display, target: &mut Frame) {
        match self {
            State::Loading(s) => s.redraw(display, target),
            State::Play(s) => s.redraw(display, target),
        }
    }
}

impl From<LoadingScreen> for State {
    fn from(l: LoadingScreen) -> Self {
        State::Loading(l)
    }
}

impl From<PlayScreen> for State {
    fn from(p: PlayScreen) -> Self {
        State::Play(p)
    }
}
