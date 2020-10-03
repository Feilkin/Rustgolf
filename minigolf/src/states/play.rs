//! play :)

use crate::states::load::Assets;
use mela::asset::tilemap::{Tilemap, Orthogonal};
use std::cell::RefCell;

pub struct Play {
    assets: Assets,
    level: Tilemap<Orthogonal>,

}