use bracket_lib::prelude::*;
use serde::{Serialize, Deserialize};
use specs::*;
use std::cmp::{max, min};

use crate::points_of_interest::PointsOfInterest;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub entities_tiles: Vec<Vec<Entity>>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    max_idx: usize,
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

    pub fn new_map_rooms_and_corridors(width: i32, height: i32) -> (Vec<Rect>, Map) {
        let max_idx = (width * height) as usize;
        let mut map = Map {
            tiles: vec![TileType::Wall; max_idx],
            revealed_tiles: vec![false; max_idx],
            visible_tiles: vec![false; max_idx],
            blocked_tiles: vec![false; max_idx],
            entities_tiles: vec![Vec::new(); max_idx],
            rooms: Vec::new(),
            width: width,
            height: height,
            max_idx: max_idx,
        };

        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 40;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);
                if !rooms.is_empty() {
                    let Point { x: new_x, y: new_y } = new_room.center();
                    let Point {
                        x: prev_x,
                        y: prev_y,
                    } = rooms[rooms.len() - 1].center();

                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                rooms.push(new_room);
            }
        }

        (rooms, map)
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

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked_tiles[i] = *tile == TileType::Wall;
        }
    }

    fn wall_glyph(&self, pos: Point) -> FontCharType {
        if pos.x < 1 || pos.x > self.width-2 || pos.y < 1 || pos.y > self.height-2 { return 35; }
        let mut mask : u8 = 0;

        if self.is_revealed_and_wall(pos + Point {x: 0, y:-1}) { mask +=1; }
        if self.is_revealed_and_wall(pos + Point {x: 0, y: 1}) { mask +=2; }
        if self.is_revealed_and_wall(pos + Point {x:-1, y: 0}) { mask +=4; }
        if self.is_revealed_and_wall(pos + Point {x: 1, y: 0}) { mask +=8; }
    
        match mask {
            0 => { 9 } // Pillar because we can't see neighbors
            1 => { 186 } // Wall only to the north
            2 => { 186 } // Wall only to the south
            3 => { 186 } // Wall to the north and south
            4 => { 205 } // Wall only to the west
            5 => { 188 } // Wall to the north and west
            6 => { 187 } // Wall to the south and west
            7 => { 185 } // Wall to the north, south and west
            8 => { 205 } // Wall only to the east
            9 => { 200 } // Wall to the north and east
            10 => { 201 } // Wall to the south and east
            11 => { 204 } // Wall to the north, south and east
            12 => { 205 } // Wall to the east and west
            13 => { 202 } // Wall to the east, west, and south
            14 => { 203 } // Wall to the east, west, and north
            15 => { 206 }  // ╬ Wall on all sides
            _ => { 35 } // We missed one?
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
                match tile {
                    TileType::Floor => {
                        glyph = to_cp437('.');
                        fg = RGB::from_f32(0.0, 0.5, 0.5);
                    }
                    TileType::Wall => {
                        glyph = map.wall_glyph(map.idx_xy(idx));
                        fg = RGB::from_f32(0., 1.0, 0.);
                    }
                }
                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale()
                }

                let bg = if poi.contains(map.idx_xy(idx)) { RGB::named(BLUE) } else { RGB::from_f32(0., 0., 0.) };
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
