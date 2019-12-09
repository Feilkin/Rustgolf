//! impl game

mod states;

use glutin::event::Event;
use glutin::event_loop::ControlFlow;

use crate::game::Playable;
use crate::game::assets::Asset;
use crate::game::assets::image::Image;
use std::collections::HashMap;
use crate::game::state::State;
use std::rc::Rc;
use std::cell::RefCell;

pub(crate) struct Minigolf {
    states: HashMap<String, Rc<RefCell<dyn State>>>,
    current_state: Rc<RefCell<dyn State>>,
}

impl Minigolf {
    pub fn new(display: &glium::Display) -> Minigolf {
        let mut states: HashMap<String, Rc<RefCell<dyn State>>> = HashMap::new();

        states.insert("loading_screen".to_owned(), Rc::new(RefCell::new(states::LoadingScreen::new(display))));
        let mut current_state = states.get("loading_screen").expect("what happened to loading screen?").clone();

        current_state.borrow_mut().focus(display);

        Minigolf {
            states,
            current_state,
        }
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
        println!(" - redrawing game");
        self.current_state.borrow_mut().redraw(display, target);
    }
}