use crate::{components::*, map::{Map, Decal}};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, positions, mut map) = data;

        for (mut stats, damage, position) in (&mut stats, &damage, &positions).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
            map.decal_tiles.insert(position.pos, Decal::blood());
        }

        damage.clear();
    }
}
