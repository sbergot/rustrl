use std::collections::HashMap;

use bracket_lib::prelude::{DistanceAlg, Point};
use specs::*;

use crate::{
    components::*,
    gui::gui_handlers::ItemUsage,
    resources::{PlayerEntity, PlayerPos},
};

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

pub fn get_inventory_options(ecs: &World) -> Vec<(String, Entity)> {
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

pub fn get_cells_in_range(ecs: &World, range: i32) -> Vec<Point> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let player_pos = ecs.read_resource::<PlayerPos>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(player_entity.entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for tile in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(player_pos.pos, *tile);
            if distance <= range as f32 {
                available_cells.push(*tile);
            }
        }
    }
    available_cells
}
