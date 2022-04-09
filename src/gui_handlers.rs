use bracket_lib::prelude::{BTerm, Point};
use specs::*;

use crate::{
    components::*,
    gui::{self, ItemMenuResult},
    player::PlayerEntity,
    state::RunState,
};

#[derive(PartialEq, Copy, Clone)]
pub enum UiScreen {
    Inventory,
    DropItem,
    Targeting { range: i32, item: Entity },
}

pub trait UiHandler<T> {
    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<T>);
    fn handle(&self, ecs: &mut World, input: T) -> RunState;
}

pub struct InventoryHandler {}

impl UiHandler<Entity> for InventoryHandler {
    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
        gui::show_inventory(ecs, ctx)
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

pub struct DropItemHandler {}

impl UiHandler<Entity> for DropItemHandler {
    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
        gui::drop_item_menu(ecs, ctx)
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

pub struct TargetingHandler {
    range: i32,
    item: Entity,
}

impl UiHandler<Point> for TargetingHandler {
    fn show(&self, ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Point>) {
        gui::ranged_target(ecs, ctx, self.range)
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
