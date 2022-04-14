use bracket_lib::prelude::{BTerm, Point};
use specs::*;

use crate::{
    components::*,
    gui::game_ui::*,
    player::PlayerEntity,
    state::RunState,
};

#[derive(PartialEq, Copy, Clone)]
pub enum UiScreen {
    Inventory,
    DropItem,
    Targeting { range: i32, item: Entity },
}

pub fn run_screen(ecs: &mut World, ctx: &mut BTerm, screen: UiScreen) -> Option<RunState> {
    match screen {
        UiScreen::Inventory => (InventoryHandler {}).run_handler(ecs, ctx),
        UiScreen::DropItem => (DropItemHandler {}).run_handler(ecs, ctx),
        UiScreen::Targeting { range, item } => (TargetingHandler { range, item }).run_handler(ecs, ctx)
    }
}


trait UiHandler {
    type Output;
    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Self::Output>);
    fn handle(&self, ecs: &mut World, input: Self::Output) -> RunState;

    fn run_handler(&self, ecs: &mut World, ctx: &mut BTerm) -> Option<RunState> {
        let (menuresult, output) = self.show(ecs, ctx);
        match menuresult {
                ItemMenuResult::Cancel => Some(RunState::AwaitingInput),
                ItemMenuResult::NoResponse => None,
                ItemMenuResult::Selected => {
                Some(self.handle(ecs, output.unwrap()))
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct InventoryHandler {}

impl UiHandler for InventoryHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
        show_inventory(ecs, ctx)
    }

    fn handle(&self, ecs: &mut World, input: Entity) -> RunState {
        let is_ranged = ecs.read_storage::<Ranged>();
        let is_item_ranged = is_ranged.get(input);
        if let Some(is_item_ranged) = is_item_ranged {
            RunState::ShowUi {
                screen: UiScreen::Targeting {
                    range: is_item_ranged.range,
                    item: input,
                },
            }
        } else {
            let mut intent = ecs.write_storage::<WantsToUseItem>();
            intent
                .insert(
                    ecs.fetch::<PlayerEntity>().entity,
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

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
        drop_item_menu(ecs, ctx)
    }

    fn handle(&self, ecs: &mut World, input: Entity) -> RunState {
        let mut intent = ecs.write_storage::<WantsToDropItem>();
        intent
            .insert(
                ecs.fetch::<PlayerEntity>().entity,
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
}

impl UiHandler for TargetingHandler {
    type Output = Point;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Point>) {
        ranged_target(ecs, ctx, self.range)
    }

    fn handle(&self, ecs: &mut World, input: Point) -> RunState {
        let mut intent = ecs.write_storage::<WantsToUseItem>();
        intent
            .insert(
                ecs.fetch::<PlayerEntity>().entity,
                WantsToUseItem {
                    item: self.item,
                    target: Some(input),
                },
            )
            .expect("Unable to insert intent");
        RunState::PlayerTurn
    }
}
