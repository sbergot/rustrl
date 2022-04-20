use bracket_lib::prelude::*;
use specs::*;

use crate::{
    components::*,
    gui::{components::*, game_ui::*},
    input::*,
    player::{PlayerEntity, PlayerPos},
    points_of_interest::PointsOfInterest,
    state::RunState, queries::*,
};

#[derive(PartialEq, Copy, Clone)]
pub enum ItemUsage {
    Use,
    Drop,
    Equip,
    Unequip,
}

#[derive(PartialEq, Copy, Clone)]
pub enum UiScreen {
    Inventory,
    UseItem {
        item: Entity,
    },
    DropItem,
    RemoveItem,
    Targeting {
        range: i32,
        item: Entity,
        selection: Point,
    },
    Examine {
        selection: Point,
    },
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult<T> {
    Cancel,
    NoResponse,
    Selected { result: T },
}

pub fn read_input_selection<T: Copy>(key: Option<VirtualKeyCode>, options: &Vec<(String, T)>) -> ItemMenuResult<T> {
    let count = options.len();

    match key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return ItemMenuResult::Selected { result: options[selection as usize].1 };
                }
                ItemMenuResult::NoResponse
            }
        },
    }
}

pub fn run_screen(ecs: &mut World, ctx: &mut BTerm, screen: UiScreen) -> Option<RunState> {
    match screen {
        UiScreen::Inventory => (InventoryHandler {}).run_handler(ecs, ctx),
        UiScreen::DropItem => (DropItemHandler {}).run_handler(ecs, ctx),
        UiScreen::Targeting {
            range,
            item,
            selection,
        } => (TargetingHandler {
            range,
            item,
            selection,
        })
        .run_handler(ecs, ctx),
        UiScreen::RemoveItem => (EquippedItemHandler {}).run_handler(ecs, ctx),
        UiScreen::Examine { selection } => (ExamineHandler { selection }).run_handler(ecs, ctx),
        UiScreen::UseItem { item } => (UseItemHandler { item }).run_handler(ecs, ctx),
    }
}

trait UiHandler {
    type Output;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm);

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output>;

    fn handle(&self, ecs: &mut World, input: Self::Output) -> RunState;

    fn run_handler(&self, ecs: &mut World, ctx: &mut BTerm) -> Option<RunState> {
        self.show(ecs, ctx);
        let menuresult = self.read_input(ecs, ctx);
        match menuresult {
            ItemMenuResult::Cancel => Some(RunState::AwaitingInput),
            ItemMenuResult::NoResponse => None,
            ItemMenuResult::Selected { result } => Some(self.handle(ecs, result)),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct InventoryHandler {}

impl UiHandler for InventoryHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        let options = get_inventory_options(ecs);
        show_selection(ctx, "Inventory", &options);
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_inventory_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, _ecs: &mut World, input: Entity) -> RunState {
        RunState::ShowUi {
            screen: UiScreen::UseItem { item: input },
        }
    }
}

fn try_use_item(ecs: &mut World, input: Entity) -> RunState {
    let is_ranged = ecs.read_storage::<Ranged>();
    let is_item_ranged = is_ranged.get(input);
    if let Some(is_item_ranged) = is_item_ranged {
        let player_pos = ecs.read_resource::<PlayerPos>();
        RunState::ShowUi {
            screen: UiScreen::Targeting {
                range: is_item_ranged.range,
                item: input,
                selection: player_pos.pos,
            },
        }
    } else {
        let mut intent = ecs.write_storage::<WantsToUseItem>();
        intent
            .insert(
                ecs.read_resource::<PlayerEntity>().entity,
                WantsToUseItem {
                    item: input,
                    target: None,
                },
            )
            .expect("Unable to insert intent");
        RunState::PlayerTurn
    }
}

#[derive(PartialEq, Copy, Clone)]
struct DropItemHandler {}

impl UiHandler for DropItemHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        let options = get_inventory_options(ecs);
        show_selection(ctx, "Drop Which Item?", &options)
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_inventory_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, ecs: &mut World, input: Entity) -> RunState {
        let mut intent = ecs.write_storage::<WantsToDropItem>();
        intent
            .insert(
                ecs.read_resource::<PlayerEntity>().entity,
                WantsToDropItem { item: input },
            )
            .expect("Unable to insert intent");
        RunState::PlayerTurn
    }
}

#[derive(PartialEq, Copy, Clone)]
struct TargetingHandler {
    range: i32,
    item: Entity,
    selection: Point,
}

enum LookCommand {
    Inspect(Point),
    Select(Point),
}

impl UiHandler for TargetingHandler {
    type Output = LookCommand;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        ctx.print_color(
            5,
            0,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            "Select Target:",
        );

        let available_cells = get_cells_in_range(ecs, self.range);

        for tile in available_cells.iter() {
            ctx.set_bg(tile.x, tile.y, RGB::named(BLUE));
        }

        let pos = self.selection;
        let color = if available_cells.contains(&pos) {
            RGB::named(CYAN)
        } else {
            RGB::named(RED)
        };
        ctx.set_bg(pos.x, pos.y, color);
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let input = map_all(ctx.key, &[map_direction, map_look_commands]);
        match input {
            None => ItemMenuResult::NoResponse,
            Some(cmd) => match cmd {
                Command::Direction { direction } => ItemMenuResult::Selected {
                    result: LookCommand::Inspect(self.selection + get_direction_offset(direction)),
                },
                Command::NextTarget => {
                    let poi = ecs.read_resource::<PointsOfInterest>();
                    let next_pos = poi.get_next(self.selection);
                    match next_pos {
                        Some(next_pos) => ItemMenuResult::Selected {
                            result: LookCommand::Inspect(next_pos),
                        },
                        None => ItemMenuResult::NoResponse,
                    }
                }
                Command::Validate => ItemMenuResult::Selected {
                    result: LookCommand::Select(self.selection),
                },
                Command::Cancel => ItemMenuResult::Cancel,
                _ => ItemMenuResult::NoResponse,
            },
        }
    }

    fn handle(&self, ecs: &mut World, input: LookCommand) -> RunState {
        match input {
            LookCommand::Select(selection) => {
                let mut intent = ecs.write_storage::<WantsToUseItem>();
                intent
                    .insert(
                        ecs.read_resource::<PlayerEntity>().entity,
                        WantsToUseItem {
                            item: self.item,
                            target: Some(selection),
                        },
                    )
                    .expect("Unable to insert intent");
                RunState::PlayerTurn
            }
            LookCommand::Inspect(point) => RunState::ShowUi {
                screen: UiScreen::Targeting {
                    range: self.range,
                    item: self.item,
                    selection: point,
                },
            },
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct EquippedItemHandler {}

impl UiHandler for EquippedItemHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        let options = get_equipped_options(ecs);
        show_selection(ctx, "Remove Which Item?", &options)
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_equipped_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, _ecs: &mut World, input: Entity) -> RunState {
        RunState::ShowUi {
            screen: UiScreen::UseItem { item: input },
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct ExamineHandler {
    selection: Point,
}

impl UiHandler for ExamineHandler {
    type Output = Point;

    fn show(&self, _ecs: &mut World, ctx: &mut BTerm) {
        ctx.print_color(5, 0, RGB::named(YELLOW), RGB::named(BLACK), "Examine mode");

        let pos = self.selection;
        ctx.set_bg(pos.x, pos.y, RGB::named(CYAN));
        draw_tooltips(_ecs, ctx, pos);
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let input = map_all(ctx.key, &[map_direction, map_look_commands]);
        match input {
            None => ItemMenuResult::NoResponse,
            Some(cmd) => match cmd {
                Command::Direction { direction } => ItemMenuResult::Selected {
                    result: self.selection + get_direction_offset(direction),
                },
                Command::NextTarget => {
                    let poi = ecs.read_resource::<PointsOfInterest>();
                    let next_pos = poi.get_next(self.selection);
                    match next_pos {
                        Some(next_pos) => ItemMenuResult::Selected { result: next_pos },
                        None => ItemMenuResult::NoResponse,
                    }
                }
                Command::Validate => ItemMenuResult::NoResponse,
                Command::Cancel => ItemMenuResult::Cancel,
                _ => ItemMenuResult::NoResponse,
            },
        }
    }

    fn handle(&self, _ecs: &mut World, input: Point) -> RunState {
        RunState::ShowUi {
            screen: UiScreen::Examine { selection: input },
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct UseItemHandler {
    item: Entity,
}

impl UiHandler for UseItemHandler {
    type Output = ItemUsage;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        let options = get_usage_options(ecs, self.item);
        show_selection(ctx, "Pick action", &options);
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_usage_options(ecs, self.item);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, ecs: &mut World, input: ItemUsage) -> RunState {
        let player_entity = ecs.read_resource::<PlayerEntity>().entity;
        match input {
            ItemUsage::Drop => {
                ecs.write_storage::<WantsToDropItem>()
                    .insert(player_entity, WantsToDropItem { item: self.item })
                    .expect("could not insert intent");
            }
            ItemUsage::Equip => {
                return try_use_item(ecs, self.item);
            }
            ItemUsage::Unequip => {
                ecs.write_storage::<WantsToRemoveItem>()
                    .insert(player_entity, WantsToRemoveItem { item: self.item })
                    .expect("could not insert intent");
            }
            ItemUsage::Use => {
                return try_use_item(ecs, self.item);
            }
        }
        RunState::PlayerTurn
    }
}
