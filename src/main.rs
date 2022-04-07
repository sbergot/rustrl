mod components;
mod constants;
mod map;
mod player;
mod state;
mod gui;
mod gamelog;
mod spawner;
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
