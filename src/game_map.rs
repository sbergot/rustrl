use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};
use specs::*;
use std::collections::HashMap;

use crate::map::Map;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    Door,
    Window,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Decal {
    pub color: RGB,
}

impl Decal {
    pub fn blood() -> Decal {
        Decal {
            color: RGB::named(RED),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct GameMap {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub decal_tiles: HashMap<usize, Decal>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub entities_tiles: Vec<Vec<Entity>>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
}

impl BaseMap for GameMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(Point { x: x - 1, y: y }) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(Point { x: x + 1, y: y }) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(Point { x: x, y: y - 1 }) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(Point { x: x, y: y + 1 }) {
            exits.push((idx + w, 1.0))
        };

        if self.is_exit_valid(Point { x: x - 1, y: y - 1 }) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(Point { x: x + 1, y: y - 1 }) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(Point { x: x - 1, y: y + 1 }) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(Point { x: x + 1, y: y + 1 }) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.idx_xy(idx1);
        let p2 = self.idx_xy(idx2);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for GameMap {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map for GameMap {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl GameMap {
    fn is_exit_valid(&self, pos: Point) -> bool {
        if pos.x < 1 || pos.x > self.width - 1 || pos.y < 1 || pos.y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(pos);
        !self.blocked_tiles[idx as usize]
    }

    pub fn clear_content_index(&mut self) {
        for content in self.entities_tiles.iter_mut() {
            content.clear();
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked_tiles[i] = *tile == TileType::Wall;
        }
    }

    pub fn is_revealed_and_wall(&self, pos: Point) -> bool {
        let idx = self.xy_idx(pos);
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
    }
}
