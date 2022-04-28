use crate::{game_map::*, map::Map};
use bracket_lib::prelude::*;
use std::{
    cmp::{max, min},
    collections::HashMap,
};

pub struct BuildingsGenerator {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    max_idx: usize,
}

impl Map for BuildingsGenerator {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BuildingsGenerator {
    pub fn new(width: i32, height: i32) -> BuildingsGenerator {
        let max_idx = (width * height) as usize;
        BuildingsGenerator {
            tiles: vec![TileType::Floor; max_idx],
            width,
            height,
            max_idx,
        }
    }

    pub fn new_map_buildings(&mut self) -> (Vec<Rect>, GameMap) {
        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 5;
        const MIN_SIZE: i32 = 12;
        const MAX_SIZE: i32 = 20;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.height - h - 1) - 1;
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                self.place_building(&new_room);
                rooms.push(new_room);
            }
        }

        let map = GameMap {
            tiles: self.tiles.clone(),
            revealed_tiles: vec![false; self.max_idx],
            visible_tiles: vec![false; self.max_idx],
            blocked_tiles: vec![false; self.max_idx],
            entities_tiles: vec![Vec::new(); self.max_idx],
            decal_tiles: HashMap::new(),
            rooms: Vec::new(),
            width: self.width,
            height: self.height,
        };

        (rooms, map)
    }

    fn place_building(&mut self, building: &Rect) {
        self.place_horizontal_line(building.x1, building.x2, building.y1, TileType::Wall);
        self.place_horizontal_line(building.x1, building.x2, building.y2, TileType::Wall);
        self.place_vertical_line(building.y1, building.y2, building.x1, TileType::Wall);
        self.place_vertical_line(building.y1, building.y2, building.x2, TileType::Wall);
    }

    fn place_horizontal_line(&mut self, x1: i32, x2: i32, y: i32, tile: TileType) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(Point { x, y });
            if idx > 0 && idx < self.max_idx {
                self.tiles[idx as usize] = tile;
            }
        }
    }

    fn place_vertical_line(&mut self, y1: i32, y2: i32, x: i32, tile: TileType) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(Point { x, y });
            if idx > 0 && idx < self.max_idx {
                self.tiles[idx as usize] = tile;
            }
        }
    }
}
