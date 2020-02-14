//! Asset loading and such

pub use spritesheet::Spritesheet;

pub use crate::assets::image::Image;

pub(crate) mod image;
pub(crate) mod spritesheet;
pub mod tilemap;

pub enum Asset {
    Image(Image),
}

#[derive(Debug)]
pub enum AssetError {
    FileNotFound,
    TextureCreationError,
}
