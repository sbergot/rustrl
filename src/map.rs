use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};
use specs::*;
use std::collections::HashMap;

use crate::points_of_interest::PointsOfInterest;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Decal {
    color: RGB,
}

impl Decal {
    pub fn blood() -> Decal {
        Decal {
            color: RGB::named(RED),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub decal_tiles: HashMap<Point, Decal>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub entities_tiles: Vec<Vec<Entity>>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
}

impl BaseMap for Map {
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
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    pub fn xy_idx(&self, pos: Point) -> usize {
        ((pos.y * self.width) + pos.x) as usize
    }

    pub fn idx_xy(&self, idx: usize) -> Point {
        let x = (idx as i32) % self.width;
        let y = (idx as i32) / self.width;
        Point { x, y }
    }

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

    fn wall_glyph(&self, pos: Point) -> FontCharType {
        if pos.x < 1 || pos.x > self.width - 2 || pos.y < 1 || pos.y > self.height - 2 {
            return 35;
        }
        let mut mask: u8 = 0;

        if self.is_revealed_and_wall(pos + Point { x: 0, y: -1 }) {
            mask += 1;
        }
        if self.is_revealed_and_wall(pos + Point { x: 0, y: 1 }) {
            mask += 2;
        }
        if self.is_revealed_and_wall(pos + Point { x: -1, y: 0 }) {
            mask += 4;
        }
        if self.is_revealed_and_wall(pos + Point { x: 1, y: 0 }) {
            mask += 8;
        }

        match mask {
            0 => 9,    // Pillar because we can't see neighbors
            1 => 186,  // Wall only to the north
            2 => 186,  // Wall only to the south
            3 => 186,  // Wall to the north and south
            4 => 205,  // Wall only to the west
            5 => 188,  // Wall to the north and west
            6 => 187,  // Wall to the south and west
            7 => 185,  // Wall to the north, south and west
            8 => 205,  // Wall only to the east
            9 => 200,  // Wall to the north and east
            10 => 201, // Wall to the south and east
            11 => 204, // Wall to the north, south and east
            12 => 205, // Wall to the east and west
            13 => 202, // Wall to the east, west, and south
            14 => 203, // Wall to the east, west, and north
            15 => 206, // ╬ Wall on all sides
            _ => 35,   // We missed one?
        }
    }

    fn is_revealed_and_wall(&self, pos: Point) -> bool {
        let idx = self.xy_idx(pos);
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
    }

    pub fn draw_map(ecs: &World, ctx: &mut BTerm) {
        let map = ecs.read_resource::<Map>();
        let poi = ecs.read_resource::<PointsOfInterest>();

        let mut y = 0;
        let mut x = 0;
        for (idx, tile) in map.tiles.iter().enumerate() {
            if map.revealed_tiles[idx] {
                let glyph;
                let mut fg;
                let pos = map.idx_xy(idx);
                match tile {
                    TileType::Floor => {
                        glyph = to_cp437('.');
                        fg = RGB::from_f32(0.0, 0.5, 0.5);
                    }
                    TileType::Wall => {
                        glyph = map.wall_glyph(pos);
                        fg = RGB::from_f32(0., 1.0, 0.);
                    }
                }
                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale()
                }

                let bg = if poi.contains(pos) {
                    RGB::named(BLUE)
                } else if let Some(decal) = map.decal_tiles.get(&pos) {
                    decal.color
                } else {
                    RGB::from_f32(0., 0., 0.)
                };
                ctx.set(x, y, fg, bg, glyph);
            }

            // Move the coordinates
            x += 1;
            if x > map.width - 1 {
                x = 0;
                y += 1;
            }
        }
    }
}
