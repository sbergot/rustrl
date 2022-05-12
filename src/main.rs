mod actions;
mod components;
mod constants;
mod entity_containers;
mod game_display;
mod game_map;
mod gamelog;
mod gui;
mod input;
mod map;
mod map_generation;
mod random_table;
mod resources;
mod scenes;
mod spawner;
mod state;
mod systems;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};
use constants::*;

const FONT_BYTES: &[u8] = include_bytes!("../resources/Nice_curses_12x12.png");

fn main() -> BError {
    bracket_lib::prelude::EMBED.lock().add_resource("resources/font.png".to_string(), FONT_BYTES);
    let gs = scenes::SceneHandler::new();
    let mut context = BTermBuilder::new()
        .with_fps_cap(30.0)
        .with_font("font.png", 12, 12)
        .with_simple_console(MAP_WIDTH, MAP_HEIGHT + UI_HEIGHT, "font.png")
        .with_title("RustRL 2")
        .with_fullscreen(true)
        .build()?;
    context.with_post_scanlines(true);
    main_loop(context, gs)
}
