//! Asset loading and such

pub(crate) mod image;
pub(crate) mod spritesheet;

pub use crate::assets::image::Image;
pub use spritesheet::Spritesheet;

pub enum Asset {
    Image(Image),
}

#[derive(Debug)]
pub enum AssetError {
    FileNotFound,
    TextureCreationError,
}
