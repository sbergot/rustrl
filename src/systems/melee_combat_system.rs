use specs::prelude::*;

use crate::{components::*, gamelog::GameLog};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            names,
            combat_stats,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut inflict_damage,
            mut wants_melee,
        ) = data;

        for (wants_melee, name, stats, entity) in
            (&wants_melee, &names, &combat_stats, &entities).join()
        {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonuses, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in
                        (&entities, &defense_bonuses, &equipped).join()
                    {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(
                        0,
                        stats.power + offensive_bonus - target_stats.defense - defensive_bonus,
                    );

                    if damage == 0 {
                        log.log(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.log(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
