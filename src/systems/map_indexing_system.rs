use specs::prelude::*;

use crate::{
    components::{BlocksTile, Position},
    game_map::GameMap,
    map::Map,
};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, GameMap>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (position, entity) in (&position, &entities).join() {
            let idx = map.xy_idx(position.pos);

            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked_tiles[idx] = true;
            }

            map.entities_tiles[idx].push(entity);
        }
    }
}
