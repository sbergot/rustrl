mod components;
mod constants;
mod gamelog;
mod gui;
mod gui_handlers;
mod map;
mod player;
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
