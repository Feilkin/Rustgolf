//! impl game

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glutin::event::Event;
use glutin::event_loop::ControlFlow;
use mela::{glium, glutin};

use mela;
use mela::assets::{Asset, Image};
use mela::game::Playable;
use mela::state::State;

use crate::states;

pub(crate) struct Minigolf {
    states: Vec<Rc<RefCell<dyn State>>>,
    current_state: Rc<RefCell<dyn State>>,
}

impl Minigolf {
    pub fn new() -> Minigolf {
        let states = states::states();

        let current_state = states
            .get(0)
            .expect("what happened to loading screen?")
            .clone();

        Minigolf {
            states,
            current_state,
        }
    }

    pub fn state<T: AsRef<str>>(&self, name: T) -> Rc<RefCell<dyn State>> {
        for state in &self.states {
            let state_borrowed: &RefCell<dyn State> = state.borrow();
            if state_borrowed.borrow().name().eq(name.as_ref()) {
                return state.clone();
            }
        }

        panic!("state {} not found!", name.as_ref());
    }

    pub fn current_state(&self) -> Rc<RefCell<dyn State>> {
        self.current_state.clone()
    }
}

impl Playable for Minigolf {
    fn update(&mut self, display: &glium::Display) {
        //self.current_state.update();
    }

    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow> {
        None
    }

    fn redraw(&self, display: &glium::Display, target: &mut glium::Frame) {
        self.current_state.borrow_mut().redraw(display, target);
    }
}
