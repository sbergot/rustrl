use std::collections::HashMap;

use bracket_lib::prelude::{DistanceAlg, Point};
use specs::*;

use crate::{
    components::*,
    gui::gui_handlers::ItemUsage,
    player::{PlayerEntity, PlayerPos},
};

pub fn get_usage_options(ecs: &mut World, item: Entity) -> Vec<(String, ItemUsage)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut options = Vec::new();

    let consumable = ecs.read_storage::<Consumable>();
    if consumable.contains(item) {
        options.push(("use".to_string(), ItemUsage::Use));
    }

    let equippable = ecs.read_storage::<Equippable>();
    let equipped_storage = ecs.read_storage::<Equipped>();
    let equipped = equipped_storage.get(item);

    if let Some(equipped) = equipped {
        if equipped.owner == player_entity.entity {
            options.push(("unequip".to_string(), ItemUsage::Unequip));
        }
    } else {
        if equippable.contains(item) {
            options.push(("equip".to_string(), ItemUsage::Equip));
        }
        options.push(("drop".to_string(), ItemUsage::Drop));
    }

    options
}

pub fn get_inventory_options(ecs: &mut World) -> Vec<(String, Entity)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();
    let options: Vec<(String, Entity)> = (&backpack, &names, &entities)
        .join()
        .filter(|(item, _name, _e)| item.owner == player_entity.entity)
        .map(|(_i, name, entity)| (name.name.clone(), entity))
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

pub fn get_equipped_options(ecs: &mut World) -> Vec<(String, Entity)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<Equipped>();
    let entities = ecs.entities();
    let options: Vec<(String, Entity)> = (&entities, &backpack, &names)
        .join()
        .filter(|(_entity, pack, _name)| pack.owner == player_entity.entity)
        .map(|(entity, _pack, name)| (name.name.clone(), entity))
        .collect();
    options
}

pub fn get_cells_in_range(ecs: &mut World, range: i32) -> Vec<Point> {
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
