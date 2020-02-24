//! Asset loading and such

pub use spritesheet::Spritesheet;

pub use crate::assets::image::Image;
use std::io;
use std::io::{Error, ErrorKind};

pub(crate) mod image;
pub(crate) mod spritesheet;
pub mod tilemap;

pub enum Asset {
    Image(Image),
}

#[derive(Debug)]
pub enum AssetError {
    TextureCreationError,
    IoError(io::Error),
    XmlError(serde_xml_rs::Error),
    JsonError(serde_json::Error),
    ImageError(image::ImageError),
}

impl From<io::Error> for AssetError {
    fn from(io_err: io::Error) -> Self {
        AssetError::IoError(io_err)
    }
}

impl From<serde_xml_rs::Error> for AssetError {
    fn from(err: serde_xml_rs::Error) -> Self {
        AssetError::XmlError(err)
    }
}

impl From<serde_json::Error> for AssetError {
    fn from(err: serde_json::Error) -> Self {
        AssetError::JsonError(err)
    }
}

impl From<image::ImageError> for AssetError {
    fn from(ie: image::ImageError) -> AssetError {
        match ie {
            image::ImageError::IoError(err) => err.into(),
            _ => AssetError::ImageError(ie),
        }
    }
}
