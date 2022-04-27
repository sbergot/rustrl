use specs::prelude::*;

use crate::{
    components::{Player, Viewshed},
    map::Map,
    resources::PointsOfInterest,
};

pub struct PointsOfInterestSystem {}

impl<'a> System<'a> for PointsOfInterestSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteExpect<'a, PointsOfInterest>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut poi, viewshed, players) = data;
        for (_player, viewshed) in (&players, &viewshed).join() {
            poi.clear();
            for vis in viewshed.visible_tiles.iter() {
                let idx = map.xy_idx(*vis);
                if !map.entities_tiles[idx].is_empty() {
                    poi.add(*vis)
                }
            }
        }
    }
}
