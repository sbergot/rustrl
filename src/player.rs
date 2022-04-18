use crate::components::*;
use crate::gamelog::GameLog;
use crate::gui::gui_handlers::UiScreen;
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

pub enum Command {
    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
    Grab,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    SaveQuit,
}

fn map_key(key: VirtualKeyCode) -> Option<Command> {
    match key {
        VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => Some(Command::Left),
        VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => Some(Command::Right),
        VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => Some(Command::Up),
        VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => Some(Command::Down),
        VirtualKeyCode::Y => Some(Command::UpLeft),
        VirtualKeyCode::U => Some(Command::UpRight),
        VirtualKeyCode::B => Some(Command::DownLeft),
        VirtualKeyCode::N => Some(Command::DownRight),
        VirtualKeyCode::G => Some(Command::Grab),
        VirtualKeyCode::I => Some(Command::ShowInventory),
        VirtualKeyCode::D => Some(Command::ShowDropItem),
        VirtualKeyCode::R => Some(Command::ShowRemoveItem),
        VirtualKeyCode::Escape => Some(Command::SaveQuit),
        _ => None,
    }
}

pub fn player_input(world: &mut World, key: Option<VirtualKeyCode>) -> RunState {
    let cmd = key.and_then(map_key);
    match cmd {
        None => return RunState::AwaitingInput,
        Some(command) => match command {
            Command::Left => try_move_player(-1, 0, world),
            Command::Right => try_move_player(1, 0, world),
            Command::Up => try_move_player(0, -1, world),
            Command::Down => try_move_player(0, 1, world),
            Command::UpLeft => try_move_player(-1, -1, world),
            Command::UpRight => try_move_player(1, -1, world),
            Command::DownLeft => try_move_player(-1, 1, world),
            Command::DownRight => try_move_player(1, 1, world),
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
            Command::SaveQuit => return RunState::SaveGame,
        },
    }
    RunState::PlayerTurn
}

fn grab_item(ecs: &mut World) {
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
