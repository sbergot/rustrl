use crate::components::*;
use crate::state::PlayerPos;
use bracket_lib::prelude::console;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, PlayerPos>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewshed, pos, monster, name) = data;

        for (viewshed, pos, _monster, name) in (&viewshed, &pos, &monster, &name).join() {
            if viewshed.visible_tiles.contains(&player_pos.pos) {
                console::log(format!("{} shouts!", name.name));
            }
        }
    }
}
