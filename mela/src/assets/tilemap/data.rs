//! Data type definitions for import

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tileset {
    pub version: String,
    pub tiledversion: String,
    pub name: String,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub spacing: u32,
    pub tilecount: usize,
    pub columns: usize,
    pub image: Image,
    pub tile: Vec<Tile>,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    pub id: usize,
    pub objectgroup: Vec<ObjectGroup>,
}

#[derive(Debug, Deserialize)]
pub struct ObjectGroup {
    draworder: DrawOrder,
    id: usize,
    object: Vec<Object>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum DrawOrder {
    Index,
}

#[derive(Debug, Deserialize)]
pub struct Object {
    id: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}
