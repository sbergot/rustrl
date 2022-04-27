mod actions;
mod components;
mod constants;
mod entity_containers;
mod game_map;
mod gamelog;
mod gui;
mod input;
mod map;
mod map_generation;
mod player;
mod queries;
mod resources;
mod spawner;
mod state;
mod systems;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};
use constants::*;

fn main() -> BError {
    let gs = state::init_state(MAPWIDTH, MAPHEIGHT);
    let builder = BTermBuilder::simple(MAPWIDTH, MAPHEIGHT + UI_HEIGHT)?;
    let mut context = builder.with_title("Hello Minimal Bracket World").build()?;
    context.with_post_scanlines(true);
    main_loop(context, gs)
}
