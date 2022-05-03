use crate::{game_map::*, map::Map};
use bracket_lib::prelude::*;
use std::{
    cmp::{max, min},
    collections::HashMap,
};

use super::MapGenerator;

pub struct RoomsCorridorsGenerator {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    max_idx: usize,
}

impl Map for RoomsCorridorsGenerator {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl MapGenerator for RoomsCorridorsGenerator {
    fn generate(&mut self) -> GameMap {
        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 40;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

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
                self.apply_room_to_map(&new_room);
                if !rooms.is_empty() {
                    let Point { x: new_x, y: new_y } = new_room.center();
                    let Point {
                        x: prev_x,
                        y: prev_y,
                    } = rooms[rooms.len() - 1].center();

                    if rng.range(0, 2) == 1 {
                        self.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        self.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
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
            rooms: rooms,
            width: self.width,
            height: self.height,
        };

        map
    }
}

impl RoomsCorridorsGenerator {
    pub fn new(width: i32, height: i32) -> RoomsCorridorsGenerator {
        let max_idx = (width * height) as usize;
        RoomsCorridorsGenerator {
            tiles: vec![TileType::Wall; max_idx],
            width,
            height,
            max_idx,
        }
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(Point { x, y });
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(Point { x, y });
            if idx > 0 && idx < self.max_idx {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(Point { x, y });
            if idx > 0 && idx < self.max_idx {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
}
