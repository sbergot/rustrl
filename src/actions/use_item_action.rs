use bracket_lib::prelude::*;
use specs::{Entity, WorldExt, shred::Fetch};

use crate::{components::*, gamelog::GameLog, map::Map, systems::ParticleBuilder};

use super::{has_component, Action};

pub struct UseItemAction {
    pub item: Entity,
    pub target: Option<Point>,
}

impl Action for UseItemAction {
    fn run(&self, actor: Entity, ecs: &mut specs::World) {
        let mut used_item: bool = false;
        let map: Fetch<Map> = ecs.read_resource();
        let mut particle_builder = ecs.write_resource::<ParticleBuilder>();
        let is_player = has_component::<Player>(ecs, actor);
        let mut log = ecs.write_resource::<GameLog>();

        // Targeting
        let mut targets: Vec<Entity> = Vec::new();
        match self.target {
            None => {
                targets.push(actor);
            }
            Some(target) => {
                let storage = ecs.read_component::<AreaOfEffect>();
                let area_effect = storage.get(self.item);
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
                            particle_builder.request(
                                *tile_idx,
                                RGB::named(ORANGE),
                                RGB::named(BLACK),
                                to_cp437('░'),
                                200.0,
                            );
                        }
                    }
                }
            }
        }

        let mut combat_stats_storage = ecs.write_component::<CombatStats>();
        let name_storage = ecs.write_component::<Name>();
        let position_storage = ecs.write_component::<Position>();

        let storage = ecs.read_component::<ProvidesHealing>();
        let item_heals = storage.get(self.item);
        match item_heals {
            None => {}
            Some(healer) => {
                for target in targets.iter() {
                    let target_stats = combat_stats_storage.get_mut(*target);
                    if let Some(target_stats) = target_stats {
                        target_stats.heal(healer.heal_amount);
                        if is_player {
                            log.log(format!(
                                "You use the {}, healing {} hp.",
                                name_storage.get(self.item).unwrap().name,
                                healer.heal_amount
                            ));
                        }
                        used_item = true;
                    }

                    let pos = position_storage.get(*target);
                    if let Some(pos) = pos {
                        particle_builder.request(
                            pos.pos,
                            RGB::named(GREEN),
                            RGB::named(BLACK),
                            to_cp437('♥'),
                            200.0,
                        );
                    }
                }
            }
        }

        let storage = ecs.read_component::<InflictsDamage>();
        let item_damages = storage.get(self.item);
        match item_damages {
            None => {}
            Some(damage) => {
                for target in targets.iter() {
                    let target_stats = combat_stats_storage.get_mut(*target);
                    if let Some(target_stats) = target_stats {
                        target_stats.deal_damage(damage.damage);
                    }
                    if is_player {
                        let mob_name = name_storage.get(*target).unwrap();
                        let item_name = name_storage.get(self.item).unwrap();
                        log.log(format!(
                            "You use {} on {}, inflicting {} hp.",
                            item_name.name, mob_name.name, damage.damage
                        ));

                        let pos = position_storage.get(*target);
                        if let Some(pos) = pos {
                            particle_builder.request(
                                pos.pos,
                                RGB::named(RED),
                                RGB::named(BLACK),
                                to_cp437('‼'),
                                200.0,
                            );
                        }
                    }
                    used_item = true;
                }
            }
        }

        // Can it pass along confusion? Note the use of scopes to escape from the borrow checker!
        let mut add_confusion = Vec::new();
        {
            let storage = ecs.read_component::<Confusion>();
            let causes_confusion = storage.get(self.item);
            match causes_confusion {
                None => {}
                Some(confusion) => {
                    used_item = false;
                    for mob in targets.iter() {
                        add_confusion.push((*mob, confusion.turns));
                        if is_player {
                            let mob_name = name_storage.get(*mob).unwrap();
                            let item_name = name_storage.get(self.item).unwrap();
                            log.log(format!(
                                "You use {} on {}, confusing them.",
                                item_name.name, mob_name.name
                            ));

                            let pos = position_storage.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.pos,
                                    RGB::named(MAGENTA),
                                    RGB::named(BLACK),
                                    to_cp437('?'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }
        }

        let mut confused_storage = ecs.write_component::<Confused>();
        for mob in add_confusion.iter() {
            confused_storage
                .insert(mob.0, Confused { turns: mob.1 })
                .expect("Unable to insert status");
        }

        if has_component::<Consumable>(ecs, self.item) && used_item {
            let mut storage = ecs.write_storage::<Inventory>();
            let inventory = storage.get_mut(actor).unwrap();
            if let Some(index) = inventory.items.iter().position(|ent| *ent == self.item) {
                inventory.items.remove(index);
            }
    
            let entities = ecs.entities();
            entities.delete(self.item).expect("Delete failed");
        }
    }
}
