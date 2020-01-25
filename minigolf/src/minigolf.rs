//! impl game

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glutin::event::{Event, StartCause, WindowEvent};
use glutin::event_loop::ControlFlow;
use mela::{glium, glutin};

use mela;
use mela::assets::{Asset, Image};
use mela::game::{IoState, Playable};
use mela::state::State;

use crate::states;
use crate::states::LoadingScreen;
use std::time::Duration;

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
    ) -> Minigolf {
        let next_state = self
            .current_state
            .update(delta, display, ui, &self.io_state);

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
            _ => (),
        };

        None
    }

    fn redraw(&self, display: &glium::Display, target: &mut glium::Frame) {
        self.current_state.redraw(display, target);
    }
}
