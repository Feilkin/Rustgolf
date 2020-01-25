//! Aseprite exported spritesheets (.json + .png)

use crate::assets::{AssetError, Image};
use crate::gfx::{Quad, Texture};
use glium::Display;
use serde::Deserialize;
use serde_json::Error as JsonError;
use std::fs::File;
use std::io::{BufReader, Error as IoError};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SpritesheetError {
    FileError(IoError),
    DataError(JsonError),
    AssetError(AssetError),
}

impl From<IoError> for SpritesheetError {
    fn from(err: IoError) -> Self {
        SpritesheetError::FileError(err)
    }
}

impl From<JsonError> for SpritesheetError {
    fn from(err: JsonError) -> Self {
        SpritesheetError::DataError(err)
    }
}

impl From<AssetError> for SpritesheetError {
    fn from(err: AssetError) -> Self {
        SpritesheetError::AssetError(err)
    }
}

#[derive(Debug)]
pub struct Spritesheet {
    data: Data,
    texture: Texture,
    quads: Vec<Quad>,
}

impl Spritesheet {
    pub fn from_file<P: AsRef<Path>>(
        filename: P,
        display: &Display,
    ) -> Result<Spritesheet, SpritesheetError> {
        let file = File::open(filename.as_ref())?;
        let reader = BufReader::new(file);

        let data: Data = serde_json::from_reader(reader)?;
        let image = Image::from_file(
            filename
                .as_ref()
                .parent()
                .and_then(|p| Some(p.join(Path::new(&data.meta.image))))
                .unwrap_or(PathBuf::from(&data.meta.image)),
            display,
        )?;

        let source_size = image.dimensions();
        let quads = data
            .frames
            .iter()
            .map(|f| {
                Quad::new(
                    [f.frame.x as f32, f.frame.y as f32],
                    [f.frame.w as f32, f.frame.h as f32],
                    source_size,
                )
            })
            .collect();

        Ok(Spritesheet {
            texture: image.into(),
            quads,
            data,
        })
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn quad(&self, index: usize) -> &Quad {
        &self.quads[index]
    }
}

// JSON data stuff

#[derive(Debug, Deserialize)]
pub struct Data {
    pub frames: Vec<Frame>,
    pub meta: Meta,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    filename: String,
    pub frame: FrameSizePos,
    pub rotated: bool,
    pub trimmed: bool,
    sprite_source_size: FrameSizePos,
    pub source_size: FrameSize,
    duration: u32,
}

// TODO: names

#[derive(Debug, Deserialize)]
pub struct FrameSizePos {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct FrameSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub app: String,
    pub version: String,
    pub image: String,
    // TODO: use enum here for formats maybe?
    format: String,
    size: Size,
    scale: String,
    frame_tags: Vec<FrameTag>,
    layers: Vec<Layer>,
}

#[derive(Debug, Deserialize)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Debug, Deserialize)]
struct FrameTag {
    name: String,
    from: usize,
    to: usize,
    direction: Direction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layer {
    name: String,
    opacity: u8,
    blend_mode: BlendMode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BlendMode {
    Normal,
}
