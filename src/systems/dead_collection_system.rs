use crate::{components::*, gamelog::GameLog};
use specs::prelude::*;

pub struct DeadCollection {}

impl<'a> System<'a> for DeadCollection {
    type SystemData = (
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (combat_stats, players, names, mut logs, entities) = data;
        let mut dead: Vec<Entity> = Vec::new();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            logs.log(format!("{} is dead", &victim_name.name));
                        }
                    }
                    Some(_) => {
                        logs.log("You die".to_string());
                    }
                }
                dead.push(entity);
            }
        }

        for victim in dead {
            entities.delete(victim).expect("Unable to delete");
        }
    }
}
