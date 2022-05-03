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
mod resources;
mod scenes;
mod spawner;
mod state;
mod systems;
mod random_table;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};
use constants::*;

fn main() -> BError {
    let gs = scenes::SceneHandler::new();
    let builder = BTermBuilder::simple(MAPWIDTH, MAPHEIGHT + UI_HEIGHT)?;
    let mut context = builder.with_title("RustRL").build()?;
    context.with_post_scanlines(true);
    main_loop(context, gs)
}
