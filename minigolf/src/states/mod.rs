//! gamestates
mod multiplay;
mod play;

pub enum Wrapper {
    Play(Play),
    Multiplay(Multiplay),
}

impl DebugDrawable for Wrapper {}

impl State for Wrapper {
    type Wrapper = Self;

    fn name(&self) -> &str {
        "Wrapper"
    }

    fn update(
        self,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        match self {
            Wrapper::Play(s) => s.update(delta, io_state, render_ctx, debug_ctx),
            Wrapper::Multiplay(s) => s.update(delta, io_state, render_ctx, debug_ctx),
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        match self {
            Wrapper::Play(s) => s.redraw(render_ctx, debug_ctx),
            Wrapper::Multiplay(s) => s.redraw(render_ctx, debug_ctx),
        }
    }
}

use crate::physics::Wall;
use mela::debug::{DebugContext, DebugDrawable};
use mela::game::IoState;
use mela::gfx::RenderContext;
use mela::nalgebra::Point2;
use mela::state::State;
pub use multiplay::Multiplay;
pub use play::Play;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub fn walls() -> Rc<RefCell<Vec<Wall<f64>>>> {
    Rc::new(RefCell::new(vec![
        Wall {
            start: Point2::new(4., 4.),
            end: Point2::new(4., 716.),
        },
        Wall {
            start: Point2::new(4., 716.),
            end: Point2::new(1276., 716.),
        },
        Wall {
            start: Point2::new(1276., 716.),
            end: Point2::new(1276., 4.),
        },
        Wall {
            start: Point2::new(1276., 4.),
            end: Point2::new(4., 4.),
        },
        Wall {
            start: Point2::new(63.0, 374.0),
            end: Point2::new(383.0, 689.0),
        },
        Wall {
            start: Point2::new(383.0, 689.5),
            end: Point2::new(826.0, 688.5),
        },
        Wall {
            start: Point2::new(826.0, 689.0),
            end: Point2::new(1201.0, 314.0),
        },
        Wall {
            start: Point2::new(1201.0, 313.0),
            end: Point2::new(920.0, 32.0),
        },
        Wall {
            start: Point2::new(920.0, 32.0),
            end: Point2::new(679.0, 273.0),
        },
        Wall {
            start: Point2::new(679.0, 274.0),
            end: Point2::new(861.0, 456.0),
        },
        Wall {
            start: Point2::new(861.0, 456.0),
            end: Point2::new(800.0, 517.0),
        },
        Wall {
            start: Point2::new(799.0, 517.0),
            end: Point2::new(722.0, 440.0),
        },
        Wall {
            start: Point2::new(722.0, 440.0),
            end: Point2::new(571.0, 591.0),
        },
        Wall {
            start: Point2::new(570.0, 591.0),
            end: Point2::new(481.0, 502.0),
        },
        Wall {
            start: Point2::new(481.0, 502.0),
            end: Point2::new(648.0, 335.0),
        },
        Wall {
            start: Point2::new(648.0, 335.0),
            end: Point2::new(378.0, 65.0),
        },
        Wall {
            start: Point2::new(377.0, 65.0),
            end: Point2::new(68.0, 374.0),
        },
    ]))
}
