use crate::actions::*;
use crate::components::*;
use crate::map::Map;
use crate::player::{PlayerEntity, PlayerPos};
use crate::state::RunState;
use bracket_lib::prelude::*;
use specs::prelude::*;

pub struct MonsterAI {}

type SystemData<'a> = (
    ReadExpect<'a, Map>,
    ReadExpect<'a, PlayerPos>,
    ReadExpect<'a, PlayerEntity>,
    ReadExpect<'a, RunState>,
    ReadStorage<'a, Viewshed>,
    ReadStorage<'a, Monster>,
    WriteStorage<'a, Confused>,
    WriteStorage<'a, Position>,
    Entities<'a>,
);

impl<'a> MonsterAI {
    pub fn run(&mut self, world: &mut World) {
        let mut actions: Vec<(Entity, Box<dyn Action>)> = Vec::new();

        {
            let (
                map,
                player_pos,
                player_entity,
                run_state,
                viewshed,
                monster,
                mut confused,
                mut pos,
                entities,
            ): SystemData = world.system_data();

            if *run_state != RunState::MonsterTurn {
                return;
            }

            for (viewshed, pos, _monster, entity) in
                (&viewshed, &mut pos, &monster, &entities).join()
            {
                let action = get_monster_action(
                    &mut confused,
                    entity,
                    viewshed,
                    player_pos.pos,
                    pos,
                    player_entity.entity,
                    &map,
                );
                if let Some(action) = action {
                    actions.push((entity, action));
                }
            }
        }

        for (entity, action) in actions.iter() {
            action.run(*entity, world)
        }
    }
}

fn get_monster_action(
    confused: &mut WriteStorage<Confused>,
    entity: Entity,
    viewshed: &Viewshed,
    player_pos: Point,
    pos: &mut Position,
    player_entity: Entity,
    map: &Map,
) -> AnyAction {
    let mut can_act = true;
    let is_confused = confused.get_mut(entity);
    if let Some(i_am_confused) = is_confused {
        i_am_confused.turns -= 1;
        if i_am_confused.turns < 1 {
            confused.remove(entity);
        }
        can_act = false;
    }
    if !viewshed.visible_tiles.contains(&player_pos) {
        can_act = false;
    }
    if can_act {
        let distance = DistanceAlg::Pythagoras.distance2d(pos.pos, player_pos);
        if distance < 1.5 {
            let action = MeleeAction {
                target: player_entity,
            };
            return Some(Box::new(action));
        }
        let path = a_star_search(
            map.xy_idx(pos.pos) as i32,
            map.xy_idx(player_pos) as i32,
            &*map,
        );
        if path.success && path.steps.len() > 1 {
            let target = Point {
                x: path.steps[1] as i32 % map.width,
                y: path.steps[1] as i32 / map.width,
            };
            let action = MoveAction { target };
            return Some(Box::new(action));
        }
    }

    None
}
