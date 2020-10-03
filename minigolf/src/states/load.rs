//! asset loading

use mela::asset::tilemap::{Tilemap, Orthogonal};
use mela::state::State;
use mela::gfx::RenderContext;
use mela::game::IoState;
use std::time::Duration;
use mela::debug::{DebugContext, DebugDrawable};

pub struct Assets {
    pub levels: Vec<Tilemap<Orthogonal>>
}

pub struct Loading {
    assets: Option<Assets>
}

impl Loading {
    pub fn new() -> Loading {
        Loading {
            assets: None
        }
    }
}

impl State for Loading {
    type Wrapper = Self;

    fn name(&self) -> &str {
        "Loading"
    }

    fn update(self, delta: Duration, io_state: &IoState, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) -> Self::Wrapper {
        if let Some(mut assets) = self.assets {
            for layer in assets.levels[0].layers_mut() {
                layer.update(render_ctx);
            }

            return Loading {
                assets: Some(assets)
            };
        }

        let levels = vec![
            Tilemap::from_file("assets/maps/debug/01.json", render_ctx).unwrap(),
        ];

        let assets = Assets {
            levels
        };

        Loading {
            assets: Some(assets)
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        let camera = mela::nalgebra::Matrix4::new_orthographic(0., 1920., 1080., 0., 0.001, 100.0);

        for layer in self.assets.as_ref().unwrap().levels[0].layers() {
            layer.draw(&camera, render_ctx);
        }
    }
}

impl DebugDrawable for Loading {}