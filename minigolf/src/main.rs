//! My mini golf game all rights reserved

use better_panic;
use imgui_glium_renderer::Renderer as ImguiRenderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use replace_with::replace_with_or_abort;

use mela;
use mela::{glium, glutin, imgui};

mod components;
mod minigolf;
mod states;
mod systems;
mod world;

use crate::states::LoadingScreen;
use glium::DrawError::InstancesCountMismatch;
use imgui::{FontConfig, FontSource};
use minigolf::Minigolf;
use std::time::Instant;

fn main() {
    better_panic::install();

    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Rust golf test")
        .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(2);
    let display = glium::Display::new(wb, cb, &el).unwrap();

    // Set up imgui stuff
    let mut imgui = imgui::Context::create();
    // TODO: clipboard
    let mut winit_platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        winit_platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
    }

    // Set up imgui fonts
    let hidpi_factor = winit_platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: font_size,
            ..FontConfig::default()
        }),
    }]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    let mut imgui_renderer =
        ImguiRenderer::init(&mut imgui, &display).expect("Failed to initialize ImGui renderer");

    let mut game = Minigolf::new();

    use mela::state::State;
    game.current_state_mut().load(&display);

    let mut last_imgui_frame = Instant::now();
    let mut last_frame = Instant::now();
    let delta_time_target = std::time::Duration::from_nanos(16);

    el.run(move |event, window_target, control_flow| {
        use glutin::event::{Event, StartCause, WindowEvent};
        use glutin::event_loop::ControlFlow;
        use mela::game::Playable;

        // pass event to imgui
        winit_platform.handle_event(imgui.io_mut(), display.gl_window().window(), &event);

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { .. },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::MouseInput { .. },
                ..
            } => {
                game.push_event(&event);
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let this_frame_start = Instant::now();

                // update imgui
                let io = imgui.io_mut();
                winit_platform
                    .prepare_frame(io, &display.gl_window().window())
                    .expect("Failed to start frame");
                last_imgui_frame = io.update_delta_time(last_imgui_frame);
                let mut ui = imgui.frame();

                // update game
                let delta = Instant::now() - last_frame;
                last_frame = Instant::now();
                replace_with_or_abort(&mut game, |game| game.update(delta, &display, &mut ui));

                // render game
                use glium::Surface;
                let mut target = display.draw();
                target.clear_color(1.0, 0.0, 1.0, 1.0);

                game.redraw(&display, &mut target);

                let draw_data = ui.render();
                imgui_renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");

                target.finish().unwrap();

                let this_frame_took = Instant::now() - this_frame_start;

                if this_frame_took < delta_time_target {
                    let next_frame_time =
                        std::time::Instant::now() + (delta_time_target - this_frame_took);
                    *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
                } else {
                    *control_flow = glutin::event_loop::ControlFlow::Poll
                }
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                display.gl_window().window().request_redraw();
            }
            // we poll when last frame took too long
            Event::NewEvents(StartCause::Poll) => {
                display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    });
}
