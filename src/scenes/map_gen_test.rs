use bracket_lib::prelude::*;

use crate::{
    constants::{MAPHEIGHT, MAPWIDTH},
    game_display::draw_map,
    game_map::GameMap,
    input::{map_look_commands, Command},
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
    gen_type: MapGenType,
}

impl Scene for MapGenTestScene {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        self.draw(ctx);
        match self.read_input(ctx) {
            Some(cmd) => match cmd {
                Command::Cancel => SceneSignal::Load(SceneType::MainMenu),
                Command::Validate => {
                    self.map = Self::gen_map(self.gen_type);
                    SceneSignal::None
                }
                _ => SceneSignal::None,
            }
            None => SceneSignal::None,
        }
    }
}

impl MapGenTestScene {
    pub fn new(gen_type: MapGenType) -> Self {
        let map = Self::gen_map(gen_type);
        let poi = PointsOfInterest::new();
        MapGenTestScene { map, poi, gen_type }
    }

    fn gen_map(gen_type: MapGenType) -> GameMap {
        let mut map = match gen_type {
            MapGenType::Rooms => {
                let mut generator = RoomsCorridorsGenerator::new(MAPWIDTH, MAPHEIGHT);
                let map = generator.generate();
                map
            }
            MapGenType::Buildings => {
                let mut generator = BuildingsGenerator::new(MAPWIDTH, MAPHEIGHT);
                let map = generator.generate();
                map
            }
        };

        map.visible_tiles.fill(true);
        map.revealed_tiles.fill(true);

        map
    }

    fn read_input(&mut self, ctx: &BTerm) -> Option<Command> {
        ctx.key.and_then(map_look_commands)
    }

    fn draw(&self, ctx: &mut BTerm) {
        ctx.cls();
        draw_map(&self.map, &self.poi, ctx)
    }
}
