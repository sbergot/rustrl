use specs::*;

use crate::{
    components::{Position, WantsToMove, Player},
    map::Map, player::PlayerPos,
};

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, WantsToMove>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, PlayerPos>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut wants_moves, mut positions, mut map, mut player_pos, players) = data;
        for (wants_move, position, player) in (&wants_moves, &mut positions, players.maybe()).join() {
            let target_idx = map.xy_idx(wants_move.target);
            if !map.blocked_tiles[target_idx] {
                let pos_idx = map.xy_idx(position.pos);
                map.blocked_tiles[pos_idx] = false;
                map.blocked_tiles[target_idx] = true;
                position.pos = wants_move.target;

                if let Some(_p) = player {
                    player_pos.pos = wants_move.target;
                }
            }
        }
        wants_moves.clear();
    }
}
