use bracket_lib::prelude::{field_of_view, Point};
use specs::prelude::*;

use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::player::PlayerEntity;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == player_entity.entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        ReadStorage<'a, Confusion>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, WantsToUseItem>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, Confused>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            player_entity,
            mut gamelog,
            entities,
            players,
            names,
            healings,
            inflict_damage,
            area_of_effect,
            confusion,
            consumables,
            equippables,
            mut equipped,
            mut wants_use_items,
            mut suffer_damage,
            mut combat_stats,
            mut confused,
            mut backpack,
        ) = data;

        for (wants_use_item, player) in (&wants_use_items, players.maybe()).join() {
            let mut used_item: bool = false;

            // Targeting
            let mut targets: Vec<Entity> = Vec::new();
            match wants_use_item.target {
                None => {
                    targets.push(player_entity.entity);
                }
                Some(target) => {
                    let area_effect = area_of_effect.get(wants_use_item.item);
                    match area_effect {
                        None => {
                            // Single target in tile
                            let idx = map.xy_idx(target);
                            for mob in map.entities_tiles[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            // AoE
                            let mut blast_tiles = field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(*tile_idx);
                                for mob in map.entities_tiles[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            let item_equippable = equippables.get(wants_use_item.item);
            if let Some(can_equip) = item_equippable {
                let target_slot = can_equip.slot;
                let target = targets[0];

                // Remove any items the target has in the item's slot
                let mut to_unequip: Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        to_unequip.push(item_entity);
                        if let Some(_p) = player {
                            gamelog.entries.push(format!("You unequip {}.", name.name));
                        }
                    }
                }
                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack
                        .insert(*item, InBackpack { owner: target })
                        .expect("Unable to insert backpack entry");
                }

                // Wield the item
                equipped
                    .insert(
                        wants_use_item.item,
                        Equipped {
                            owner: target,
                            slot: target_slot,
                        },
                    )
                    .expect("Unable to insert equipped component");
                backpack.remove(wants_use_item.item);
                if let Some(_p) = player {
                    gamelog.entries.push(format!(
                        "You equip {}.",
                        names.get(wants_use_item.item).unwrap().name
                    ));
                }
            }

            let item_heals = healings.get(wants_use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if let Some(_p) = player {
                                gamelog.entries.push(format!(
                                    "You use the {}, healing {} hp.",
                                    names.get(wants_use_item.item).unwrap().name,
                                    healer.heal_amount
                                ));
                            }
                            used_item = true;
                        }
                    }
                }
            }

            let item_damages = inflict_damage.get(wants_use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if let Some(_p) = player {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(wants_use_item.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name.name, mob_name.name, damage.damage
                            ));
                        }
                        used_item = true;
                    }
                }
            }

            // Can it pass along confusion? Note the use of scopes to escape from the borrow checker!
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confusion.get(wants_use_item.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if let Some(_p) = player {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(wants_use_item.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them.",
                                    item_name.name, mob_name.name
                                ));
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused
                    .insert(mob.0, Confused { turns: mob.1 })
                    .expect("Unable to insert status");
            }

            let consumable = consumables.get(wants_use_item.item);
            if consumable != None && used_item {
                entities.delete(wants_use_item.item).expect("Delete failed");
            }
        }

        wants_use_items.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropped_pos: Point;
            {
                let dropper_pos = positions.get(entity).unwrap();
                dropped_pos = dropper_pos.pos;
            }
            positions
                .insert(to_drop.item, Position { pos: dropped_pos })
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == player_entity.entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}
