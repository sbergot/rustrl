use crate::components::*;
use crate::map::Map;
use crate::player::{PlayerEntity, PlayerPos};
use crate::state::RunState;
use bracket_lib::prelude::*;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, PlayerPos>,
        ReadExpect<'a, PlayerEntity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            player_pos,
            player_entity,
            run_state,
            mut viewshed,
            mut pos,
            monster,
            mut wants_to_melee,
            entities,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (viewshed, pos, _monster, entity) in
            (&mut viewshed, &mut pos, &monster, &entities).join()
        {
            if viewshed.visible_tiles.contains(&player_pos.pos) {
                let distance =
                    DistanceAlg::Pythagoras.distance2d(pos.pos, player_pos.pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: player_entity.entity,
                            },
                        )
                        .expect("Unable to insert attack");
                    return;
                }
                let path = a_star_search(
                    map.xy_idx(pos.pos) as i32,
                    map.xy_idx(player_pos.pos) as i32,
                    &*map,
                );
                if path.success && path.steps.len() > 1 {
                    pos.pos.x = path.steps[1] as i32 % map.width;
                    pos.pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
