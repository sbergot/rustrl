use bracket_lib::prelude::*;

use crate::{scenes::{Scene, SceneSignal}, state::{State, init_state}, constants::{MAPWIDTH, MAPHEIGHT}, systems::load_game};

pub struct GameScene<'a, 'b> {
    state: State<'a, 'b>,
}

impl<'a, 'b> Scene for GameScene<'a, 'b> {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        self.state.tick(ctx)
    }
}

impl<'a, 'b> GameScene<'a, 'b> {
    pub fn new_game() -> GameScene<'a, 'b> {
        let state = init_state(MAPWIDTH, MAPHEIGHT);
        GameScene { state: state }
    }

    pub fn load_game() -> GameScene<'a, 'b> {
        let mut state = init_state(MAPWIDTH, MAPHEIGHT);
        load_game(&mut state.ecs);
        GameScene { state: state }
    }
}