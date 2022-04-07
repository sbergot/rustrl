use bracket_lib::prelude::Point;
use specs::prelude::*;

use crate::{components::*, player};
use crate::gamelog::GameLog;
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
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut gamelog,
            entities,
            mut wants_drink,
            players,
            names,
            potions,
            consumables,
            mut combat_stats,
        ) = data;

        for (drink, stats, consumable, player) in (&wants_drink, &mut combat_stats, consumables.maybe(), players.maybe()).join() {
            let potion = potions.get(drink.potion);
            match potion {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    if let Some(_p) = player {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            names.get(drink.potion).unwrap().name,
                            potion.heal_amount
                        ));
                    }
                    if let Some(_c) = consumable {
                        entities.delete(drink.potion).expect("Delete failed");
                    }
                }
            }
        }

        wants_drink.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, PlayerEntity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropped_pos : Point;
            {
                let dropper_pos = positions.get(entity).unwrap();
                dropped_pos = dropper_pos.pos;
            }
            positions.insert(to_drop.item, Position{ pos : dropped_pos }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == player_entity.entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}