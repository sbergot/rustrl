use bracket_lib::prelude::Rect;

use crate::game_map::GameMap;

pub mod rooms_corridors_generator;
pub mod buildings_generator;

pub trait MapGenerator {
    fn generate(&mut self) -> (Vec<Rect>, GameMap);
}