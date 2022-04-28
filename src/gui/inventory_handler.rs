use std::collections::HashMap;

use bracket_lib::prelude::BTerm;
use specs::{Entity, World, WorldExt};

use crate::{
    components::{Inventory, Name},
    game_display::UiSignal,
    resources::PlayerEntity,
};

use super::{
    components::show_selection,
    gui_handlers::{read_input_selection, ItemMenuResult, UiHandler, UiScreen},
};

#[derive(PartialEq, Copy, Clone)]
pub struct InventoryHandler {}

impl UiHandler for InventoryHandler {
    type Output = Entity;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        let options = get_inventory_options(ecs);
        show_selection(ctx, "Inventory", &options);
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_inventory_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, _ecs: &World, input: Entity) -> UiSignal {
        UiSignal::UpdateScreen(UiScreen::UseItem { item: input })
    }
}

fn get_inventory_options(ecs: &World) -> Vec<(String, Entity)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let storage = ecs.read_storage::<Inventory>();
    let player_inventory = storage.get(player_entity.entity).unwrap();
    let options: Vec<(String, Entity)> = player_inventory
        .items
        .iter()
        .map(|entity| (names.get(*entity).unwrap().name.clone(), *entity))
        .collect();

    let mut count_dict = HashMap::<String, i32>::new();
    let mut entity_dict = HashMap::<String, Entity>::new();
    for (name, e) in options.iter() {
        entity_dict.entry(name.clone()).or_insert(*e);
        let counter = count_dict.entry(name.clone()).or_insert(0);
        *counter += 1;
    }

    let mut new_options: Vec<(String, Entity)> = count_dict
        .iter()
        .map(|(name, count)| (format!("{} ({})", *name, *count), entity_dict[name]))
        .collect();

    new_options.sort();
    new_options
}
