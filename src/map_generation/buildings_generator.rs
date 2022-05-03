use crate::{game_map::*, map::Map, random_table::RandomTable};
use bracket_lib::prelude::*;
use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
};

use super::MapGenerator;

const EXT_IDX: i32 = -1;

#[derive(Clone)]
struct NeighBor {
    idx: i32,
    shared_wall: Vec<Point>,
}

pub struct BuildingsGenerator {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    max_idx: usize,
    rng: RandomNumberGenerator,
}

impl Map for BuildingsGenerator {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl MapGenerator for BuildingsGenerator {
    fn generate(&mut self) -> GameMap {
        let buildings = self.generate_buildings();

        let mut rooms: Vec<Rect> = Vec::new();

        for building in buildings.iter() {
            let mut building_rooms = self.generate_interior(*building);
            rooms.append(&mut building_rooms);
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

impl BuildingsGenerator {
    pub fn new(width: i32, height: i32) -> BuildingsGenerator {
        let max_idx = (width * height) as usize;
        let rng = RandomNumberGenerator::new();
        BuildingsGenerator {
            tiles: vec![TileType::Floor; max_idx],
            width,
            height,
            max_idx,
            rng,
        }
    }

    fn generate_buildings(&mut self) -> Vec<Rect> {
        let mut buildings: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 20;
        for _ in 0..MAX_ROOMS {
            let new_building = self.roll_room();
            let mut ok = true;
            for other_building in buildings.iter() {
                if new_building.intersect(&expand_rect(*other_building, 3)) {
                    ok = false
                }
            }
            if ok {
                self.place_building(&new_building);
                buildings.push(new_building);
            }
        }
        buildings
    }

    fn place_building(&mut self, building: &Rect) {
        self.place_horizontal_line(building.x1, building.x2, building.y1, TileType::Wall);
        self.place_horizontal_line(building.x1, building.x2, building.y2, TileType::Wall);
        self.place_vertical_line(building.y1, building.y2, building.x1, TileType::Wall);
        self.place_vertical_line(building.y1, building.y2, building.x2, TileType::Wall);
    }

    fn place_horizontal_line(&mut self, x1: i32, x2: i32, y: i32, tile: TileType) {
        for point in horizontal_line(x1, x2, y) {
            self.place_point(point, tile)
        }
    }

    fn place_vertical_line(&mut self, y1: i32, y2: i32, x: i32, tile: TileType) {
        for point in vertical_line(y1, y2, x) {
            self.place_point(point, tile)
        }
    }

    fn place_point(&mut self, point: Point, tile: TileType) {
        let idx = self.xy_idx(point);
        if idx > 0 && idx < self.max_idx {
            self.tiles[idx as usize] = tile;
        }
    }

    fn roll_room(&mut self) -> Rect {
        const MIN_SIZE: i32 = 24;
        const MAX_SIZE: i32 = 40;

        let w = self.rng.range(MIN_SIZE, MAX_SIZE);
        let h = self.rng.range(MIN_SIZE, MAX_SIZE);
        let x = self.rng.range(1, self.width - w - 1);
        let y = self.rng.range(1, self.height - h - 1);
        let new_room = Rect::with_size(x, y, w, h);
        new_room
    }

    fn generate_interior(&mut self, building: Rect) -> Vec<Rect> {
        let (room1, room2) = self.split_room(building);
        let mut rooms = vec![room1, room2];
        for _i in 0..3 {
            let room = self.take_random_room(&mut rooms);
            if Self::is_too_small(room) {
                rooms.push(room);
            } else {
                let (room1, room2) = self.split_room(room);
                rooms.push(room1);
                rooms.push(room2);
            }
        }

        self.connect_rooms(&rooms, building);

        rooms
    }

    fn connect_rooms(&mut self, rooms: &Vec<Rect>, building: Rect) {
        let neighbor_graph = scan_for_neighbors(rooms, building);

        // randomly walk the neighbor graph to ensure connectivity
        self.ensure_random_connectivity(&neighbor_graph);

        self.add_random_doors(&neighbor_graph);

        let ext_neighbors = neighbor_graph.get(&EXT_IDX).unwrap();
        for _i in 0..5 {
            let connection = ext_neighbors.get(self.rng.range(0, ext_neighbors.len())).unwrap();
            let wall = connection
                .shared_wall
                .get(self.rng.range(0, connection.shared_wall.len()))
                .unwrap();
            if self.tiles[self.xy_idx(*wall)] == TileType::Wall {
                self.place_point(*wall, TileType::Window);
            }
        }
    }

    fn add_random_doors(&mut self, neighbor_graph: &HashMap<i32, Vec<NeighBor>>) {
        let rooms_vec: Vec<i32> = neighbor_graph.keys().map(|i| *i).collect();
        for _i in 0..8 {
            let room_idx = rooms_vec.get(self.rng.range(0, rooms_vec.len())).unwrap();
            let neighbors = neighbor_graph.get(&room_idx).unwrap();
            let connection = neighbors.get(self.rng.range(0, neighbors.len())).unwrap();
            if connection
                .shared_wall
                .iter()
                .all(|point| self.tiles[self.xy_idx(*point)] == TileType::Wall)
            {
                let wall = connection
                    .shared_wall
                    .get(self.rng.range(0, connection.shared_wall.len()))
                    .unwrap();
                self.place_point(*wall, TileType::Door);
            }
        }
    }

    fn ensure_random_connectivity(&mut self, neighbor_graph: &HashMap<i32, Vec<NeighBor>>) {
        let mut connected = HashSet::<i32>::new();
        let mut to_connect = HashSet::<i32>::new();
        connected.insert(EXT_IDX);
        let ext_neighbors: HashSet<i32> = neighbor_graph
            .get(&EXT_IDX)
            .unwrap()
            .iter()
            .map(|n| n.idx)
            .collect();
        to_connect = to_connect.union(&ext_neighbors).map(|i| *i).collect();
        while !to_connect.is_empty() {
            let mut to_connect_vec: Vec<i32> = to_connect.iter().map(|i| *i).collect();
            let to_connect_idx = to_connect_vec.remove(self.rng.range(0, to_connect_vec.len()));

            let neighbors = neighbor_graph.get(&to_connect_idx).unwrap();
            let connected_neightbors: Vec<NeighBor> = neighbors
                .iter()
                .filter(|n| connected.contains(&n.idx))
                .map(|n| n.clone())
                .collect();

            let disconnected_neightbors: Vec<NeighBor> = neighbors
                .iter()
                .filter(|n| !connected.contains(&n.idx))
                .map(|n| n.clone())
                .collect();

            for disconnected in disconnected_neightbors.iter() {
                if !connected.contains(&disconnected.idx) {
                    to_connect.insert(disconnected.idx);
                }
            }

            let connection = connected_neightbors
                .get(self.rng.range(0, connected_neightbors.len()))
                .unwrap();

            let connection_wall = connection
                .shared_wall
                .get(self.rng.range(0, connection.shared_wall.len()))
                .unwrap();
            self.place_point(*connection_wall, TileType::Door);

            to_connect.remove(&to_connect_idx);
            connected.insert(to_connect_idx);
        }
    }

    fn is_too_small(room: Rect) -> bool {
        const MIN_SIZE: i32 = 8;
        room.x2 - room.x1 < MIN_SIZE || room.y2 - room.y1 < MIN_SIZE
    }

    fn centered_range(&mut self, min: i32, max: i32) -> i32 {
        (self.rng.range(min, max) + self.rng.range(min, max)) / 2
    }

    fn split_room(&mut self, room: Rect) -> (Rect, Rect) {
        let width = room.width();
        let height = room.height();
        let mut new_rng = self.rng.clone();
        let mut vertical_horizontal_table = RandomTable::<bool>::new(&mut new_rng)
            .add(true, width * width)
            .add(false, height * height);

        if *vertical_horizontal_table.roll() {
            //split vertically
            let new_x = self.centered_range(room.x1 + 2, room.x2 - 1);
            self.place_vertical_line(room.y1, room.y2, new_x, TileType::Wall);
            (
                Rect {
                    x1: room.x1,
                    y1: room.y1,
                    x2: new_x,
                    y2: room.y2,
                },
                Rect {
                    x1: new_x,
                    y1: room.y1,
                    x2: room.x2,
                    y2: room.y2,
                },
            )
        } else {
            //split horizontally
            let new_y = self.centered_range(room.y1 + 2, room.y2 - 1);
            self.place_horizontal_line(room.x1, room.x2, new_y, TileType::Wall);
            (
                Rect {
                    x1: room.x1,
                    y1: room.y1,
                    x2: room.x2,
                    y2: new_y,
                },
                Rect {
                    x1: room.x1,
                    y1: new_y,
                    x2: room.x2,
                    y2: room.y2,
                },
            )
        }
    }

    fn take_random_room(&mut self, rooms: &mut Vec<Rect>) -> Rect {
        // pick big rooms in priority
        let mut new_rng = self.rng.clone();
        let mut rooms_table = RandomTable::<usize>::new(&mut new_rng);
        for (i, room) in rooms.iter().enumerate() {
            rooms_table = rooms_table.add(i, room.width() * room.height());
        }

        let idx = rooms_table.roll();
        rooms.remove(*idx)
    }
}

fn horizontal_line(x1: i32, x2: i32, y: i32) -> Vec<Point> {
    let mut result = Vec::<Point>::new();
    for x in min(x1, x2)..=max(x1, x2) {
        result.push(Point { x, y });
    }
    result
}

fn vertical_line(y1: i32, y2: i32, x: i32) -> Vec<Point> {
    let mut result = Vec::<Point>::new();
    for y in min(y1, y2)..=max(y1, y2) {
        result.push(Point { x, y });
    }
    result
}

fn expand_rect(rect: Rect, size: i32) -> Rect {
    Rect {
        x1: rect.x1 - size,
        x2: rect.x2 + size,
        y1: rect.y1 - size,
        y2: rect.y2 + size,
    }
}

fn scan_for_neighbors(rooms: &Vec<Rect>, building: Rect) -> HashMap<i32, Vec<NeighBor>> {
    let mut neighbors = HashMap::<i32, Vec<NeighBor>>::new();
    for (i1, room1) in rooms.iter().enumerate() {
        for (i2, room2) in rooms.iter().enumerate() {
            if room2.x2 == room1.x1 && min(room1.y2, room2.y2) > max(room1.y1, room2.y1) {
                let shared_wall = vertical_line(
                    max(room1.y1, room2.y1) + 1,
                    min(room1.y2, room2.y2) - 1,
                    room1.x1,
                );

                let entry1 = neighbors.entry(i1 as i32).or_insert(Vec::new());
                entry1.push(NeighBor {
                    idx: i2 as i32,
                    shared_wall: shared_wall.clone(),
                });
                let entry2 = neighbors.entry(i2 as i32).or_insert(Vec::new());
                entry2.push(NeighBor {
                    idx: i1 as i32,
                    shared_wall: shared_wall,
                });
            }
            if room2.y2 == room1.y1 && min(room1.x2, room2.x2) > max(room1.x1, room2.x1) {
                let shared_wall = horizontal_line(
                    max(room1.x1, room2.x1) + 1,
                    min(room1.x2, room2.x2) - 1,
                    room1.y1,
                );

                let entry1 = neighbors.entry(i1 as i32).or_insert(Vec::new());
                entry1.push(NeighBor {
                    idx: i2 as i32,
                    shared_wall: shared_wall.clone(),
                });
                let entry2 = neighbors.entry(i2 as i32).or_insert(Vec::new());
                entry2.push(NeighBor {
                    idx: i1 as i32,
                    shared_wall: shared_wall,
                });
            }
        }
        if room1.x1 == building.x1
            || room1.x2 == building.x2
            || room1.y1 == building.y1
            || room1.y2 == building.y2
        {
            let mut shared_walls = Vec::<Point>::new();
            if room1.x1 == building.x1 {
                shared_walls.append(&mut vertical_line(room1.y1 + 1, room1.y2 - 1, room1.x1));
            }
            if room1.x2 == building.x2 {
                shared_walls.append(&mut vertical_line(room1.y1 + 1, room1.y2 - 1, room1.x2));
            }

            if room1.y1 == building.y1 {
                shared_walls.append(&mut horizontal_line(room1.x1 + 1, room1.x2 - 1, room1.y1));
            }
            if room1.y2 == building.y2 {
                shared_walls.append(&mut horizontal_line(room1.x1 + 1, room1.x2 - 1, room1.y2));
            }

            let entry1 = neighbors.entry(i1 as i32).or_insert(Vec::new());
            entry1.push(NeighBor {
                idx: EXT_IDX,
                shared_wall: shared_walls.clone(),
            });
            let entry_ext = neighbors.entry(EXT_IDX).or_insert(Vec::new());
            entry_ext.push(NeighBor {
                idx: i1 as i32,
                shared_wall: shared_walls,
            });
        }
    }

    neighbors
}
