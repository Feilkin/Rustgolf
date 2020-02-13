//! State debugger state
//!
//! Allows pausing states, and advancing them in steps.

use crate::states::State as GolfState;
use imgui_glium_renderer::glium::{Display, Frame};
use imgui_glium_renderer::imgui::Ui;
use mela::game::IoState;
use mela::state::State;
use std::default::Default;
use std::fmt::{Debug, Error, Formatter};
use std::time::Duration;
use mela::profiler;

#[derive(Default)]
struct UiState {
    step_delta: f64,
}

pub struct StateDebugger<S: State> {
    ui_state: UiState,
    inner: S,
}

impl<S: State> Debug for StateDebugger<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "StateDebugger<{}>", self.inner.name())
    }
}

impl<S: State> StateDebugger<S> {
    pub fn new(inner: S) -> StateDebugger<S> {
        StateDebugger {
            ui_state: UiState::default(),
            inner,
        }
    }
}

impl<S: State> AsRef<S> for StateDebugger<S> {
    fn as_ref(&self) -> &S {
        &self.inner
    }
}

impl<S: State> AsMut<S> for StateDebugger<S> {
    fn as_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl State for StateDebugger<GolfState> {
    type Wrapper = GolfState;

    fn name(&self) -> &str {
        "StateDebugger"
    }

    fn load(&mut self, display: &Display) {
        self.inner.load(display)
    }

    fn focus(&mut self, display: &Display) {
        self.inner.focus(display)
    }

    fn leave(&mut self, display: &Display) {
        self.inner.leave(display)
    }

    fn update(
        self,
        delta: Duration,
        display: &Display,
        ui: &mut Ui,
        io_state: &IoState,
        profiler_frame: &mut profiler::OpenFrame,
    ) -> Self::Wrapper {
        use mela::imgui::*;

        let mut run_next_frame = false;
        let mut resume_play = false;

        Window::new(im_str!("State debugger"))
            .size([300., 100.], Condition::Appearing)
            .build(&ui, || {
                if ui.button(im_str!("next frame"), [0., 0.]) {
                    run_next_frame = true;
                }
                if ui.button(im_str!("resume game"), [0., 0.]) {
                    resume_play = true;
                }
            });

        if run_next_frame {
            let inner = self.inner.update(delta, display, ui, io_state, profiler_frame);

            GolfState::StateDebugger(Box::new(StateDebugger { inner, ..self }))
        } else if resume_play {
            self.inner.into()
        } else {
            GolfState::StateDebugger(Box::new(self))
        }
    }

    fn redraw(&mut self, display: &Display, target: &mut Frame, profiler_frame: &mut profiler::OpenFrame) {
        self.inner.redraw(display, target, profiler_frame)
    }

    fn update_debug_ui(&mut self, ui: &mut mela::imgui::Ui) {
        self.inner.update_debug_ui(ui);
    }
}
