//! Stateful game states manager factory

use crate::game::IoState;
use std::time::Duration;
use crate::profiler::Profiler;
use crate::profiler;

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
        profiler_frame: &mut profiler::OpenFrame
    ) -> Self::Wrapper;
    fn redraw(&mut self, display: &glium::Display, target: &mut glium::Frame, profiler_frame: &mut profiler::OpenFrame);

    //TODO: make this #[cfg(debugger)] or something
    fn update_debug_ui(&mut self, ui: &mut imgui::Ui) {}
}
