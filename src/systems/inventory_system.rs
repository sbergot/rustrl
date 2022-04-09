use bracket_lib::prelude::Point;
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
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut gamelog,
            map,
            entities,
            mut wants_use_items,
            players,
            names,
            healings,
            inflict_damage,
            mut suffer_damage,
            consumables,
            mut combat_stats,
        ) = data;

        for (wants_use_item, stats, player) in
            (&wants_use_items, &mut combat_stats, players.maybe()).join()
        {
            let mut used_item: bool = false;

            let healing = healings.get(wants_use_item.item);
            if let Some(healing) = healing {
                stats.hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                used_item = true;
                if let Some(_p) = player {
                    gamelog.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        names.get(wants_use_item.item).unwrap().name,
                        healing.heal_amount
                    ));
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(wants_use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = wants_use_item.target.unwrap();
                    let idx = map.xy_idx(target_point);
                    used_item = false;
                    for mob in map.entities_tiles[idx].iter() {
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
