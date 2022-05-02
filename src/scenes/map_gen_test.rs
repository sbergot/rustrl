use bracket_lib::prelude::*;

use crate::{
    constants::{MAPHEIGHT, MAPWIDTH},
    game_display::draw_map,
    game_map::GameMap,
    input::map_look_commands,
    map_generation::{
        buildings_generator::BuildingsGenerator,
        rooms_corridors_generator::RoomsCorridorsGenerator, MapGenerator,
    },
    resources::PointsOfInterest,
    scenes::{Scene, SceneSignal},
};

use super::{map_gen_selection::MapGenType, SceneType};

pub struct MapGenTestScene {
    map: GameMap,
    poi: PointsOfInterest,
}

impl Scene for MapGenTestScene {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        self.draw(ctx);
        if self.read_input(ctx) {
            SceneSignal::Load(SceneType::MainMenu)
        } else {
            SceneSignal::None
        }
    }
}

impl MapGenTestScene {
    pub fn new(gen_type: MapGenType) -> Self {
        let mut map = match gen_type {
            MapGenType::Rooms => {
                let mut generator = RoomsCorridorsGenerator::new(MAPWIDTH, MAPHEIGHT);
                let (_rooms, map) = generator.generate();
                map
            }
            MapGenType::Buildings => {
                let mut generator = BuildingsGenerator::new(MAPWIDTH, MAPHEIGHT);
                let (_rooms, map) = generator.generate();
                map
            }
        };

        map.visible_tiles.fill(true);
        map.revealed_tiles.fill(true);

        let poi = PointsOfInterest::new();
        MapGenTestScene { map, poi }
    }

    fn read_input(&mut self, ctx: &BTerm) -> bool {
        ctx.key.map(map_look_commands).is_some()
    }

    fn draw(&self, ctx: &mut BTerm) {
        ctx.cls();
        draw_map(&self.map, &self.poi, ctx)
    }
}
