//! My mini golf game all rights reserved

use mela;
use mela::{glium, glutin};

mod minigolf;
mod states;

use minigolf::Minigolf;
use std::borrow::Borrow;

fn main() {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Rust golf test")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &el).unwrap();

    let mut game = Minigolf::new();

    game.current_state().borrow_mut().load(&display);

    el.run(move |event, window_target, control_flow| {
        use glutin::event::{Event, StartCause, WindowEvent};
        use glutin::event_loop::ControlFlow;
        use mela::game::Playable;

        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                // update game
                game.update(&display);

                // render game
                use glium::Surface;
                let mut target = display.draw();
                target.clear_color(1.0, 0.0, 1.0, 1.0);

                game.redraw(&display, &mut target);

                target.finish().unwrap();
            }
            _ => (),
        }
    });
}
