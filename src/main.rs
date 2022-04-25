mod components;
mod constants;
mod gamelog;
mod gui;
mod input;
mod map;
mod player;
mod points_of_interest;
mod spawner;
mod state;
mod systems;
mod queries;
mod actions;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};
use constants::*;

fn main() -> BError {
    let gs = state::init_state(MAPWIDTH, MAPHEIGHT);
    let builder = BTermBuilder::simple(MAPWIDTH, MAPHEIGHT + UI_HEIGHT)?;
    let mut context = builder.with_title("Hello Minimal Bracket World").build()?;
    context.with_post_scanlines(true);
    main_loop(context, gs)
}
