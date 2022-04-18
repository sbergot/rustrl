use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{
    components::{Player, Position, Viewshed},
    map::Map, points_of_interest::PointsOfInterest,
};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        WriteExpect<'a, PointsOfInterest>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, mut poi, mut viewshed, pos, players) = data;
        poi.clear();
        for (player, viewshed, pos) in (players.maybe(), &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(pos.pos, viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see
                if let Some(_p) = player {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(*vis);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                        if !map.entities_tiles[idx].is_empty() {
                            poi.add(*vis)
                        }
                    }
                }
                viewshed.dirty = false;
            }
        }
    }
}
