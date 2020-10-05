//! impl game

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use mela;
use mela::game::{IoState, Playable};
use mela::state::State;

use crate::states::{Play, Wrapper};
use mela::debug::{DebugContext, DebugDrawable};
use mela::gfx::RenderContext;
use mela::winit::event::{ElementState, Event, MouseButton, WindowEvent};
use mela::winit::event_loop::ControlFlow;

pub(crate) struct Minigolf {
    state: Wrapper,
    io_state: IoState,
}

impl Minigolf {
    pub fn new() -> Minigolf {
        Minigolf {
            state: Wrapper::Play(Play::new()),
            io_state: Default::default(),
        }
    }
}

impl Playable for Minigolf {
    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self {
        let Minigolf {
            mut state,
            io_state,
        } = self;

        let state = state.update(delta, &io_state, render_ctx, debug_ctx);

        Minigolf { state, io_state }
    }

    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow> {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => return Some(ControlFlow::Exit),
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

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) -> () {
        self.state.redraw(render_ctx, debug_ctx);
    }
}
