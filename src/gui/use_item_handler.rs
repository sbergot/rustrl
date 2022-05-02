use bracket_lib::prelude::*;
use specs::*;

use crate::{
    actions::*,
    components::*,
    game_display::{GameSignal, UiSignal},
    gui::components::*,
    resources::{PlayerEntity, PlayerPos}, input::{ItemMenuResult, read_input_selection},
};

use super::gui_handlers::{ItemUsage, UiHandler, UiScreen};

#[derive(PartialEq, Copy, Clone)]
pub struct UseItemHandler {
    pub item: Entity,
}

impl UiHandler for UseItemHandler {
    type Output = ItemUsage;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        let options = get_usage_options(ecs, self.item);
        show_selection(ctx, "Pick action", &options);
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_usage_options(ecs, self.item);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, ecs: &World, input: ItemUsage) -> UiSignal {
        match input {
            ItemUsage::Drop => {
                let action = DropItemAction { target: self.item };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            ItemUsage::Equip => {
                let action = EquipItemAction { target: self.item };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            ItemUsage::Unequip => {
                let action = UnequipItemAction { target: self.item };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            ItemUsage::Use => {
                return try_use_item(ecs, self.item);
            }
        }
    }
}

fn try_use_item(ecs: &World, input: Entity) -> UiSignal {
    {
        let is_ranged = ecs.read_storage::<Ranged>();
        let is_item_ranged = is_ranged.get(input);
        if let Some(is_item_ranged) = is_item_ranged {
            let player_pos = ecs.read_resource::<PlayerPos>();
            return UiSignal::UpdateScreen(UiScreen::Targeting {
                range: is_item_ranged.range,
                item: input,
                selection: player_pos.pos,
            });
        }
    }
    let action = UseItemAction {
        item: input,
        target: None,
    };
    UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
}

pub fn get_usage_options(ecs: &World, item: Entity) -> Vec<(String, ItemUsage)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut options = Vec::new();

    let consumable = ecs.read_storage::<Consumable>();
    if consumable.contains(item) {
        options.push(("use".to_string(), ItemUsage::Use));
    }

    let equippable = ecs.read_storage::<Equippable>();
    let storage = ecs.read_storage::<Equipment>();
    let player_equipment = storage.get(player_entity.entity).unwrap();
    let is_equipped = player_equipment.slots.values().any(|e| *e == item);

    if is_equipped {
        options.push(("unequip".to_string(), ItemUsage::Unequip));
    } else {
        if equippable.contains(item) {
            options.push(("equip".to_string(), ItemUsage::Equip));
        }
        options.push(("drop".to_string(), ItemUsage::Drop));
    }

    options
}
