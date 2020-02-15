//! impl game

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use glutin::event::{Event, StartCause, WindowEvent};
use glutin::event_loop::ControlFlow;

use mela;
use mela::assets::{Asset, Image};
use mela::game::{IoState, Playable};
use mela::profiler::{PopTag, Profiler, PushTag};
use mela::state::State;
use mela::{glium, glutin, profiler};

use crate::states;
use crate::states::LoadingScreen;
use mela::debug::DebugDrawable;
use imgui_glium_renderer::Renderer;

pub(crate) struct Minigolf {
    io_state: IoState,
    current_state: states::State,
}

impl Minigolf {
    pub fn new() -> Minigolf {
        Minigolf {
            current_state: states::LoadingScreen::new().into(),
            io_state: Default::default(),
        }
    }

    pub fn current_state_mut(&mut self) -> &mut states::State {
        &mut self.current_state
    }

    //pub fn switch_to<T: State + From<C>>(self) -> Minigolf {
    //    Minigolf {
    //        current_state: self.current_state.into()
    //    }
    //}
}

impl Playable for Minigolf {
    fn update(
        self,
        delta: Duration,
        display: &glium::Display,
        ui: &mut mela::imgui::Ui,
        renderer: &mut Renderer,
        profiler_frame: &mut profiler::OpenFrame,
    ) -> Minigolf {
        let mut next_state =
            self.current_state
                .update(delta, display, ui, &self.io_state, profiler_frame);

        // TODO: move this somewhere
        let debug_ui_tag = profiler_frame.push_tag("debug ui", [0.3, 0.8, 0.3, 1.0]);
        next_state.draw_debug_ui(ui, renderer);
        let _ = debug_ui_tag.pop_tag();

        Minigolf {
            current_state: next_state,
            io_state: self.io_state,
        }
    }

    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow> {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                // TODO: hidpi support?
                self.io_state.mouse_position = [position.x as f32, position.y as f32]
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state,
                        button,
                        modifiers,
                        ..
                    },
                ..
            } => {
                use mela::glutin::event::{ElementState, MouseButton};

                let button_num = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    _ => panic!("invalid button {:?}", button),
                };

                let state = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                self.io_state.mouse_buttons[button_num] = state;
            }
            _ => (),
        };

        None
    }

    fn redraw(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        profiler_frame: &mut profiler::OpenFrame,
    ) {
        self.current_state.redraw(display, target, profiler_frame)
    }
}
