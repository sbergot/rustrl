use crate::{components::*, map::{Map, Decal}};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, positions, mut map) = data;

        for (mut stats, position) in (&mut stats, &positions).join() {
            map.decal_tiles.insert(position.pos, Decal::blood());
            stats.was_hurt = false;
        }
    }
}
