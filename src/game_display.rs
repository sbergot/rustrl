use bracket_lib::prelude::*;
use specs::*;

use crate::{
    actions::AnyAction,
    components::*,
    game_map::*,
    gui::{
        game_ui::draw_ui,
        gui_handlers::{draw_screen, run_screen, UiScreen},
    },
    map::Map,
    resources::*,
};

pub struct GameDisplay {
    mode: UiScreen,
}

pub enum UiSignal {
    None,
    UpdateScreen(UiScreen),
    GameSignal(GameSignal),
}

pub enum GameSignal {
    None,
    Perform(AnyAction),
    SaveQuit,
}

impl GameDisplay {
    pub fn new() -> GameDisplay {
        GameDisplay {
            mode: UiScreen::Play,
        }
    }

    pub fn draw(&self, ecs: &World, ctx: &mut BTerm) {
        ctx.cls();
        let map = ecs.read_resource::<GameMap>();
        let poi = ecs.read_resource::<PointsOfInterest>();
        draw_map(&map, &poi, ctx);
        draw_renderables(ecs, ctx);
        draw_ui(ecs, ctx);
        draw_screen(ecs, ctx, self.mode)
    }

    pub fn read_input(&mut self, ecs: &World, ctx: &mut BTerm) -> GameSignal {
        match run_screen(ecs, ctx, self.mode) {
            UiSignal::None => GameSignal::None,
            UiSignal::UpdateScreen(screen) => {
                self.mode = screen;
                GameSignal::None
            }
            UiSignal::GameSignal(signal) => {
                self.mode = UiScreen::Play;
                signal
            }
        }
    }
}

pub fn draw_map(map: &GameMap, poi: &PointsOfInterest, ctx: &mut BTerm) {

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
                    glyph = wall_glyph(&*map, pos);
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
                TileType::Door => {
                    glyph = to_cp437('≡');
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
                TileType::Window => {
                    glyph = to_cp437('∩');
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }

            let bg = if poi.contains(pos) {
                RGB::named(BLUE)
            } else if let Some(decal) = map.decal_tiles.get(&idx) {
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

fn wall_glyph(map: &GameMap, pos: Point) -> FontCharType {
    if pos.x < 1 || pos.x > map.width - 2 || pos.y < 1 || pos.y > map.height - 2 {
        return 35;
    }
    let mut mask: u8 = 0;

    if map.is_revealed_and_wall(pos + Point { x: 0, y: -1 }) {
        mask += 1;
    }
    if map.is_revealed_and_wall(pos + Point { x: 0, y: 1 }) {
        mask += 2;
    }
    if map.is_revealed_and_wall(pos + Point { x: -1, y: 0 }) {
        mask += 4;
    }
    if map.is_revealed_and_wall(pos + Point { x: 1, y: 0 }) {
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

pub fn draw_renderables(ecs: &World, ctx: &mut BTerm) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let map = ecs.read_resource::<GameMap>();
    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render) in data {
        let idx = map.xy_idx(pos.pos);
        if map.visible_tiles[idx] {
            ctx.set(pos.pos.x, pos.pos.y, render.fg, render.bg, render.glyph)
        }
    }
}
