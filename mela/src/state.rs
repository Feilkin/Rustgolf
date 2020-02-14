//! Stateful game states manager factory

use std::time::Duration;

use crate::game::IoState;
use crate::profiler;
use crate::profiler::Profiler;

pub trait State {
    type Wrapper: State + Sized;

    fn name(&self) -> &str;
    fn load(&mut self, display: &glium::Display);
    fn focus(&mut self, display: &glium::Display);
    fn leave(&mut self, display: &glium::Display);
    fn update(
        self,
        delta: Duration,
        display: &glium::Display,
        ui: &mut imgui::Ui,
        io_state: &IoState,
        profiler_frame: &mut profiler::OpenFrame,
    ) -> Self::Wrapper;
    fn redraw(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        profiler_frame: &mut profiler::OpenFrame,
    );

    //TODO: make this #[cfg(debugger)] or something
    fn update_debug_ui(&mut self, ui: &mut imgui::Ui) {}
}
