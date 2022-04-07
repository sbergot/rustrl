mod components;
mod constants;
mod map;
mod player;
mod state;
mod visibility_system;
mod monster_ai_system;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;
mod gui;
mod gamelog;
mod spawner;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};
use constants::*;

fn main() -> BError {
    let gs = state::init_state(MAPWIDTH, MAPHEIGHT);
    let builder = BTermBuilder::simple(MAPWIDTH, MAPHEIGHT + UI_HEIGHT)?;
    let mut context = builder.with_title("Hello Minimal Bracket World").build()?;
    context.with_post_scanlines(true);
    main_loop(context, gs)
}
