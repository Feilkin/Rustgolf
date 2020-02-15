//! Mostly a wrapper around image-rs

use std::ffi::c_void;
use std::fmt::Debug;
use std::path::Path;

use gl::types::{GLint, GLuint};
use glium::texture::TextureCreationError;
use image::{DynamicImage, GenericImageView, ImageError};

use crate::assets::{Asset, AssetError};
use crate::gfx::{Quad, Texture};
use std::rc::Rc;

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

// TODO: this should probably be a newtype over image::DynamicImage
#[derive(Clone)]
pub struct Image {
    inner: DynamicImage,
    texture: Rc<Texture>,
    dimensions: (u32, u32),
}

impl Image {
    pub fn from_file<P: AsRef<Path> + Debug>(
        filename: P,
        display: &glium::Display,
    ) -> Result<Image, AssetError> {
        dbg!(&filename);

        let img = image::open(filename)?;

        let (width, height) = img.dimensions();
        let data = img.to_rgba();

        // TODO: this should probably be done in the From trait
        let img_data = glium::texture::RawImage2d::from_raw_rgba(data.to_vec(), (width, height));
        let texture = Texture::new(display, img_data)?;

        Ok(Image {
            inner: img,
            texture: Rc::new(texture),
            dimensions: (width, height),
        })
    }
    pub fn texture(&self) -> &Rc<glium::texture::Texture2d> {
        &self.texture
    }
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }
    pub fn height(&self) -> u32 {
        self.dimensions.1
    }
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}

impl From<Image> for Texture {
    fn from(img: Image) -> Self {
        Rc::try_unwrap(img.texture).unwrap()
    }
}
