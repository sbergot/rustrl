mod components;
mod constants;
mod map;
mod player;
mod state;
mod visibility_system;

use bracket_lib::prelude::{main_loop, BError, BTermBuilder};

fn main() -> BError {
    let width = 120;
    let height = 70;
    let gs = state::init_state(width, height);
    match BTermBuilder::simple(width, height) {
        Ok(builder) => {
            let context = builder.with_title("Hello Minimal Bracket World").build()?;
            main_loop(context, gs)
        }
        Err(e) => Err(e),
    }
}
