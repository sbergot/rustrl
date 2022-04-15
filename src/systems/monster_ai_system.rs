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
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Confused>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, WantsToMove>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            player_pos,
            player_entity,
            run_state,
            viewshed,
            monster,
            mut confused,
            mut pos,
            mut wants_to_melee,
            mut wants_to_move,
            entities,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (viewshed, pos, _monster, entity) in (&viewshed, &mut pos, &monster, &entities).join() {
            let mut can_act = true;

            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if !viewshed.visible_tiles.contains(&player_pos.pos) {
                can_act = false;
            }

            if can_act {
                let distance = DistanceAlg::Pythagoras.distance2d(pos.pos, player_pos.pos);
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
                    let target = Point {
                        x: path.steps[1] as i32 % map.width,
                        y: path.steps[1] as i32 / map.width,
                    };
                    wants_to_move
                        .insert(entity, WantsToMove { target })
                        .expect("Unable to insert move");
                }
            }
        }
    }
}
