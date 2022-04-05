mod components;
mod constants;
mod map;
mod player;
mod state;
mod visibility_system;
mod monster_ai_system;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};

fn main() -> BError {
    let width = 120;
    let height = 70;
    let gs = state::init_state(width, height);
    let builder = BTermBuilder::simple(width, height)?;
    let context = builder.with_title("Hello Minimal Bracket World").build()?;
    main_loop(context, gs)
}
