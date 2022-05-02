use std::cmp::{max, min};

use bracket_lib::prelude::{BTerm, Point};
use specs::*;

use crate::{
    actions::*,
    components::{CombatStats, Item, Position},
    game_display::{GameSignal, UiSignal},
    game_map::GameMap,
    gamelog::GameLog,
    input::*,
    map::Map,
    resources::{PlayerEntity, PlayerPos},
};

use super::gui_handlers::{UiHandler, UiScreen};

#[derive(PartialEq, Copy, Clone)]
pub struct PlayHandler {}

impl UiHandler for PlayHandler {
    type Output = Command;

    fn show(&self, _ecs: &World, _ctx: &mut BTerm) {}

    fn read_input(&self, _ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let cmd = map_all(ctx.key, &[map_direction, map_other_commands]);
        match cmd {
            Some(cmd) => ItemMenuResult::Selected { result: cmd },
            None => ItemMenuResult::NoResponse,
        }
    }

    fn handle(&self, ecs: &World, input: Command) -> UiSignal {
        match input {
            Command::Direction { direction } => try_move_player(direction, ecs),
            Command::Wait => UiSignal::GameSignal(GameSignal::Perform(Box::new(WaitAction {}))),
            Command::Grab => {
                let action = grab_item(ecs);
                if let Some(action) = action {
                    UiSignal::GameSignal(GameSignal::Perform(action))
                } else {
                    UiSignal::None
                }
            }
            Command::ShowInventory => return UiSignal::UpdateScreen(UiScreen::Inventory),
            Command::ShowRemoveItem => return UiSignal::UpdateScreen(UiScreen::RemoveItem),
            Command::ExamineMode => {
                let player_pos = ecs.read_resource::<PlayerPos>();
                return UiSignal::UpdateScreen(UiScreen::Examine {
                    selection: player_pos.pos,
                });
            }
            Command::SaveQuit => return UiSignal::GameSignal(GameSignal::SaveQuit),
            _ => UiSignal::None,
        }
    }
}

pub fn try_move_player(direction: Direction, ecs: &World) -> UiSignal {
    let player_entity = ecs.read_resource::<PlayerEntity>().entity;
    let action = {
        let player_pos = {
            let storage = ecs.read_storage::<Position>();
            storage.get(player_entity).unwrap().pos
        };
        let combat_stats = ecs.read_storage::<CombatStats>();
        let map = ecs.read_resource::<GameMap>();

        let offset = get_direction_offset(direction);

        get_player_action(&map, player_pos, offset, &combat_stats)
    };

    if let Some(action) = action {
        UiSignal::GameSignal(GameSignal::Perform(action))
    } else {
        UiSignal::None
    }
}

pub fn get_player_action(
    map: &GameMap,
    player_pos: Point,
    offset: Point,
    combat_stats: &ReadStorage<CombatStats>,
) -> Option<AnyAction> {
    let destination_idx = map.xy_idx(player_pos + offset);
    for potential_target in map.entities_tiles[destination_idx].iter() {
        let target = combat_stats.get(*potential_target);
        match target {
            None => {}
            Some(_t) => {
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

pub fn grab_item(ecs: &World) -> Option<AnyAction> {
    let player_pos = ecs.read_resource::<PlayerPos>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.write_resource::<GameLog>();

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
