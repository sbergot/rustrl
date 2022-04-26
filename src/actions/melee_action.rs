use bracket_lib::prelude::*;
use specs::{Entity, WorldExt};

use crate::{components::*, gamelog::GameLog, systems::ParticleBuilder};

use super::{map_equipped_items_comp, Action};

// struct WorldEntityRef<'a> {
//     world: &'a World,

//     entity: Entity,
// }

// impl WorldEntityRef<'_> {
//     fn get_component<T: Component>(&self) -> Option<&T> {
//         let storage = self.world.read_component::<T>();
//         storage.get(self.entity)
//     }
// }

pub struct MeleeAction {
    pub target: Entity,
}

impl Action for MeleeAction {
    fn run(&self, actor: Entity, ecs: &mut specs::World) {
        let mut particle_builder = ecs.write_resource::<ParticleBuilder>();
        let mut log = ecs.write_resource::<GameLog>();

        let mut combat_stats_storage = ecs.write_storage::<CombatStats>();
        let stats = combat_stats_storage.get(actor).unwrap().clone();
        let target_stats = combat_stats_storage.get_mut(self.target).unwrap();

        let name_storage = ecs.read_storage::<Name>();
        let name = name_storage.get(actor).unwrap();
        let target_name = name_storage.get(self.target).unwrap();

        let storage = ecs.read_storage::<Position>();
        let target_pos = storage.get(self.target);

        if stats.hp > 0 {
            let mut offensive_bonus = 0;
            for power_bonus in
                map_equipped_items_comp::<MeleePowerBonus, i32>(ecs, actor, |mpb| mpb.power)
            {
                offensive_bonus += power_bonus;
            }

            if target_stats.hp > 0 {
                let mut defensive_bonus = 0;

                for defense_bonus in
                    map_equipped_items_comp::<DefenseBonus, i32>(ecs, self.target, |db| db.defense)
                {
                    defensive_bonus += defense_bonus;
                }

                if let Some(pos) = target_pos {
                    particle_builder.request(
                        pos.pos,
                        RGB::named(ORANGE),
                        RGB::named(BLACK),
                        to_cp437('‼'),
                        200.0,
                    );
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
                    target_stats.deal_damage(damage);
                }
            }
        }
    }
}
