use crate::actions::*;
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

pub fn player_input(world: &mut World, key: Option<VirtualKeyCode>) -> RunState {
    let cmd = map_all(key, &[map_direction, map_other_commands]);
    match cmd {
        None => return RunState::AwaitingInput,
        Some(command) => match command {
            Command::Direction { direction } => try_move_player(direction, world),
            Command::Wait => {}
            Command::Grab => {
                let action = grab_item(world);
                if let Some(action) = action {
                    let player_entity = world.read_resource::<PlayerEntity>().entity;
                    action.run(player_entity, world);
                }
            }
            Command::ShowInventory => {
                return RunState::ShowUi {
                    screen: UiScreen::Inventory,
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

fn try_move_player(direction: Direction, ecs: &mut World) {
    let player_entity = ecs.read_resource::<PlayerEntity>().entity;
    let action = {
        let player_pos = {
            let storage = ecs.read_storage::<Position>();
            storage.get(player_entity).unwrap().pos
        };
        let combat_stats = ecs.read_storage::<CombatStats>();
        let map = ecs.read_resource::<Map>();

        let offset = get_direction_offset(direction);

        get_player_action(&map, player_pos, offset, &combat_stats)
    };

    if let Some(action) = action {
        action.run(player_entity, ecs);
    }
}

fn get_player_action(
    map: &Map,
    player_pos: Point,
    offset: Point,
    combat_stats: &ReadStorage<CombatStats>,
) -> AnyAction {
    let destination_idx = map.xy_idx(player_pos + offset);
    for potential_target in map.entities_tiles[destination_idx].iter() {
        let target = combat_stats.get(*potential_target);
        match target {
            None => {}
            Some(_t) => {
                // Attack it
                let action = MeleeAction {
                    target: *potential_target,
                };
                return Some(Box::new(action));
            }
        }
    }

    if !map.blocked_tiles[destination_idx] {
        let target = Point {
            x: min(map.width - 1, max(0, player_pos.x + offset.x)),
            y: min(map.height - 1, max(0, player_pos.y + offset.y)),
        };
        let action = MoveAction { target };
        return Some(Box::new(action));
    }

    return None;
}

fn grab_item(ecs: &mut World) -> AnyAction {
    let player_pos = ecs.read_resource::<PlayerPos>();
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
        None => {
            gamelog
                .entries
                .push("There is nothing here to pick up.".to_string());
            None
        }
        Some(item) => {
            let action = PickItemAction { target: item };
            Some(Box::new(action))
        }
    }
}
