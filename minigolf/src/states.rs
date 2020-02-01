//! spröööt :-DD

use mela::glium::{Display, Frame};
use std::cell::RefCell;
use std::rc::Rc;

use mela::state::State as MelaState;

mod loading_screen;
mod play_screen;
mod state_debugger;

pub use loading_screen::LoadingScreen;
use mela::game::IoState;
pub use play_screen::PlayScreen;
pub use state_debugger::StateDebugger;
use std::ops::Deref;
use std::time::Duration;

#[derive(Debug)]
pub enum State {
    Loading(LoadingScreen),
    Play(PlayScreen),
    StateDebugger(Box<StateDebugger<State>>),
}

impl MelaState for State {
    type Wrapper = State;

    fn name(&self) -> &str {
        match self {
            State::Loading(s) => s.name(),
            State::Play(s) => s.name(),
            State::StateDebugger(s) => s.name(),
        }
    }

    fn load(&mut self, display: &Display) {
        match self {
            State::Loading(s) => s.load(display),
            State::Play(s) => s.load(display),
            State::StateDebugger(s) => s.as_mut().load(display),
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
            State::StateDebugger(mut s) => s.update(delta, display, ui, io_state),
        }
    }

    fn redraw(&self, display: &Display, target: &mut Frame) {
        match self {
            State::Loading(s) => s.redraw(display, target),
            State::Play(s) => s.redraw(display, target),
            State::StateDebugger(s) => s.as_ref().redraw(display, target),
        }
    }

    fn update_debug_ui(&mut self, ui: &mut mela::imgui::Ui) {
        match self {
            State::Loading(s) => s.update_debug_ui(ui),
            State::Play(s) => s.update_debug_ui(ui),
            State::StateDebugger(s) => s.update_debug_ui(ui),
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

impl From<StateDebugger<State>> for State {
    fn from(s: StateDebugger<State>) -> Self {
        State::StateDebugger(Box::new(s))
    }
}
