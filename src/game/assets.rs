//! Asset loading and such

use crate::game::assets::image::Image;

pub(crate) mod image;

pub enum Asset {
    Image(Image),
}

#[derive(Debug)]
pub enum AssetError {
    FileNotFound,
    TextureCreationError,
}