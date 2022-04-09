use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::state::{RunState, State};

use bracket_lib::prelude::*;
use specs::prelude::*;
use std::cmp::{max, min};

pub struct PlayerPos {
    pub pos: Point,
}

pub struct PlayerEntity {
    pub entity: Entity,
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut wants_to_move = ecs.write_storage::<WantsToMove>();

    for (_player, pos, entity) in (&mut players, &mut positions, &entities).join() {
        let destination_idx = map.xy_idx(
            pos.pos
                + Point {
                    x: delta_x,
                    y: delta_y,
                },
        );

        for potential_target in map.entities_tiles[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(_t) => {
                    // Attack it
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                    return; // So we don't move after attacking
                }
            }
        }

        if !map.blocked_tiles[destination_idx] {
            let target = Point {
                x: min(map.width - 1, max(0, pos.pos.x + delta_x)),
                y: min(map.height - 1, max(0, pos.pos.y + delta_y)),
            };
            wants_to_move
                .insert(entity, WantsToMove { target })
                .expect("Add move failed");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs)
            }

            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::G => get_item(&mut gs.ecs),

            VirtualKeyCode::I => return RunState::ShowInventory,

            VirtualKeyCode::D => return RunState::ShowDropItem,

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<PlayerPos>();
    let player_entity = ecs.fetch::<PlayerEntity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.pos == player_pos.pos {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    player_entity.entity,
                    WantsToPickupItem {
                        collected_by: player_entity.entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}
