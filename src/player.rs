use crate::gamelog::GameLog;
use crate::gui::gui_handlers::UiScreen;
use crate::map::Map;
use crate::state::RunState;
use crate::{components::*, input::*};

use bracket_lib::prelude::*;
use specs::prelude::*;
use std::cmp::{max, min};

pub struct PlayerPos {
    pub pos: Point,
}

pub struct PlayerEntity {
    pub entity: Entity,
}

fn try_move_player(direction: Direction, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.read_resource::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut wants_to_move = ecs.write_storage::<WantsToMove>();

    let offset = get_direction_offset(direction);

    for (_player, pos, entity) in (&mut players, &mut positions, &entities).join() {
        let destination_idx = map.xy_idx(pos.pos + offset);

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
                x: min(map.width - 1, max(0, pos.pos.x + offset.x)),
                y: min(map.height - 1, max(0, pos.pos.y + offset.y)),
            };
            wants_to_move
                .insert(entity, WantsToMove { target })
                .expect("Add move failed");
        }
    }
}

pub fn player_input(world: &mut World, key: Option<VirtualKeyCode>) -> RunState {
    let cmd = map_all(key, &[map_direction, map_other_commands]);
    match cmd {
        None => return RunState::AwaitingInput,
        Some(command) => match command {
            Command::Direction { direction } => try_move_player(direction, world),
            Command::Wait => {}
            Command::Grab => grab_item(world),
            Command::ShowInventory => {
                return RunState::ShowUi {
                    screen: UiScreen::Inventory,
                }
            }
            Command::ShowDropItem => {
                return RunState::ShowUi {
                    screen: UiScreen::DropItem,
                }
            }
            Command::ShowRemoveItem => {
                return RunState::ShowUi {
                    screen: UiScreen::RemoveItem,
                }
            }
            Command::ExamineMode => {
                let player_pos = world.read_resource::<PlayerPos>();
                return RunState::ShowUi {
                    screen: UiScreen::Examine {
                        selection: player_pos.pos,
                    },
                };
            }
            Command::SaveQuit => return RunState::SaveGame,
            _ => {}
        },
    }
    RunState::PlayerTurn
}

fn grab_item(ecs: &mut World) {
    let player_pos = ecs.read_resource::<PlayerPos>();
    let player_entity = ecs.read_resource::<PlayerEntity>();
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
