//! Mostly a wrapper around image-rs

use std::path::Path;

use gl::types::{GLuint, GLint};
use image::{DynamicImage, ImageError, GenericImageView};

use crate::game::assets::{AssetError, Asset};
use std::ffi::c_void;
use glium::texture::TextureCreationError;

impl From<ImageError> for AssetError {
    fn from(ie: ImageError) -> AssetError {
        AssetError::FileNotFound
    }
}

impl From<glium::texture::TextureCreationError> for AssetError {
    fn from(tce: TextureCreationError) -> AssetError {
        AssetError::TextureCreationError
    }
}

pub struct Image {
    inner: DynamicImage,
    texture: glium::texture::Texture2d,
    dimensions: (u32, u32),
}


impl Image {
    pub fn from_file<P: AsRef<Path>>(filename: P, display: &glium::Display) -> Result<Image, AssetError> {
        let img = image::open(filename)?;
        let (width, height) = img.dimensions();
        let data = img.as_rgb8().expect("could not convert to rgba8").clone();

        let img_data = glium::texture::RawImage2d::from_raw_rgb(data.to_vec(), (width, height));
        let texture = glium::texture::Texture2d::new(display, img_data)?;

        Ok(Image {
            inner: img,
            texture,
            dimensions: (width, height)
        })
    }
    pub fn texture(&self) -> &glium::texture::Texture2d { &self.texture }
    pub fn width(&self) -> u32 { self.dimensions.0 }
    pub fn height(&self) -> u32 { self.dimensions.1 }
}