use bracket_lib::prelude::*;
use specs::*;

use crate::{
    components::*,
    gui::{components::*, game_ui::*},
    input::*,
    player::{PlayerEntity, PlayerPos},
    points_of_interest::PointsOfInterest,
    state::RunState,
};

#[derive(PartialEq, Copy, Clone)]
pub enum UiScreen {
    Inventory,
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
        UiScreen::RemoveItem => (RemoveItemHandler {}).run_handler(ecs, ctx),
        UiScreen::Examine { selection } => (ExamineHandler { selection }).run_handler(ecs, ctx),
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

    fn handle(&self, ecs: &mut World, input: Entity) -> RunState {
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

        // Draw mouse cursor
        let mouse_pos = self.selection;
        let valid_target = available_cells.contains(&mouse_pos);
        if valid_target {
            ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(CYAN));
        } else {
            ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(RED));
        }
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
struct RemoveItemHandler {}

impl UiHandler for RemoveItemHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        let options = get_equipped_options(ecs);
        show_selection(ctx, "Remove Which Item?", &options)
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_equipped_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, ecs: &mut World, input: Entity) -> RunState {
        let mut intent = ecs.write_storage::<WantsToRemoveItem>();
        intent
            .insert(
                ecs.read_resource::<PlayerEntity>().entity,
                WantsToRemoveItem { item: input },
            )
            .expect("Unable to insert intent");
        RunState::PlayerTurn
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

        // Draw mouse cursor
        let mouse_pos = self.selection;
        ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(CYAN));
        draw_tooltips(_ecs, ctx, mouse_pos);
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
            screen: UiScreen::Examine { selection: input }
        }
    }
}
