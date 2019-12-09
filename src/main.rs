//! My mini golf game all rights reserved

mod game;
mod minigolf;

use minigolf::Minigolf;

fn main() {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Rust golf test")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &el).unwrap();

    let mut game = Minigolf::new(&display);

    el.run(move |event, window_target, control_flow| {
        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;
        use game::Playable;

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            Event::EventsCleared => {
                game.update(&display);
            }
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::RedrawRequested => {
                    use glium::Surface;
                    let mut target = display.draw();
                    target.clear_color(1.0, 0.0, 1.0, 1.0);

                    println!("redrawing");
                    game.redraw(&display, &mut target);

                    target.finish().unwrap();
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => *control_flow = ControlFlow::Poll,
            },
            _ => *control_flow = ControlFlow::Poll,
        }
    });
}
