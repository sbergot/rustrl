use bracket_lib::prelude::{BTerm, Point};
use specs::*;

use crate::{components::*, gui::game_ui::*, player::{PlayerEntity, PlayerPos}, state::RunState};

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
    }
}

trait UiHandler {
    type Output;
    fn show(&self, ecs: &mut World, ctx: &mut BTerm);
    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Self::Output>);
    fn handle(&self, ecs: &mut World, input: Self::Output) -> RunState;

    fn run_handler(&self, ecs: &mut World, ctx: &mut BTerm) -> Option<RunState> {
        self.show(ecs, ctx);
        let (menuresult, output) = self.read_input(ecs, ctx);
        match menuresult {
            ItemMenuResult::Cancel => Some(RunState::AwaitingInput),
            ItemMenuResult::NoResponse => None,
            ItemMenuResult::Selected => Some(self.handle(ecs, output.unwrap())),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct InventoryHandler {}

impl UiHandler for InventoryHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        show_inventory(ecs, ctx);
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Self::Output>) {
        read_input_inventory(ecs, ctx)
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

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        drop_item_menu(ecs, ctx)
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Self::Output>) {
        read_input_inventory(ecs, ctx)
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
    selection: Point,
}

impl UiHandler for TargetingHandler {
    type Output = Point;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        show_ranged_target(ecs, ctx, self.range)
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Self::Output>) {
        read_input_ranged_target(ecs, ctx, self.range)
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

#[derive(PartialEq, Copy, Clone)]
struct RemoveItemHandler {}

impl UiHandler for RemoveItemHandler {
    type Output = Entity;

    fn show(&self, ecs: &mut World, ctx: &mut BTerm) {
        show_remove_item_menu(ecs, ctx)
    }

    fn read_input(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Self::Output>) {
        read_input_remove_item_menu(ecs, ctx)
    }

    fn handle(&self, ecs: &mut World, input: Entity) -> RunState {
        let mut intent = ecs.write_storage::<WantsToRemoveItem>();
        intent
            .insert(
                ecs.fetch::<PlayerEntity>().entity,
                WantsToRemoveItem { item: input },
            )
            .expect("Unable to insert intent");
        RunState::PlayerTurn
    }
}
