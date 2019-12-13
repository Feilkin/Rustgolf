//! Asset loading and such

pub(crate) mod image;
pub use crate::assets::image::Image;

pub enum Asset {
    Image(Image),
}

#[derive(Debug)]
pub enum AssetError {
    FileNotFound,
    TextureCreationError,
}
