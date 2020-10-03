//! My mini golf game all rights reserved

#![deny(unused_must_use)]

use std::time::{Duration, Instant};

use mela;
use minigolf::Minigolf;

use mela::application::Application;

mod minigolf;
mod states;
mod world;
mod resources;
mod components;

fn main() {
    let game = Minigolf::new();
    let app = Application::new(game, "Minigolf");

    app.run()
}
