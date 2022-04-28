use bracket_lib::prelude::BTerm;
use specs::{Entity, World, WorldExt};

use crate::{
    components::{Equipment, Name},
    game_display::UiSignal,
    resources::PlayerEntity,
};

use super::{
    components::show_selection,
    gui_handlers::{read_input_selection, ItemMenuResult, UiHandler, UiScreen},
};

#[derive(PartialEq, Copy, Clone)]
pub struct EquippedItemHandler {}

impl UiHandler for EquippedItemHandler {
    type Output = Entity;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        let options = get_equipped_options(ecs);
        show_selection(ctx, "Remove Which Item?", &options)
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_equipped_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, _ecs: &World, input: Entity) -> UiSignal {
        UiSignal::UpdateScreen(UiScreen::UseItem { item: input })
    }
}

pub fn get_equipped_options(ecs: &World) -> Vec<(String, Entity)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();

    let storage = ecs.read_storage::<Equipment>();
    let player_equipment = storage.get(player_entity.entity).unwrap();

    let options: Vec<(String, Entity)> = player_equipment
        .slots
        .values()
        .map(|entity| (names.get(*entity).unwrap().name.clone(), *entity))
        .collect();
    options
}
