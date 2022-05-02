use bracket_lib::prelude::*;

use crate::{
    gui::components::format_option,
    input::{read_input_selection, ItemMenuResult},
    scenes::{Scene, SceneSignal},
};

use super::SceneType;

#[derive(Clone, Copy, PartialEq)]
pub enum MapGenType {
    Rooms,
    Buildings,
}

#[derive(Clone)]
struct MapGenEntry {
    map_gen_type: MapGenType,
    label: &'static str,
}

pub struct MapGenSelectionScene {
    entries: Vec<MapGenEntry>,
}

impl Scene for MapGenSelectionScene {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        self.draw(ctx);
        if let Some(entry) = self.read_input(ctx) {
            SceneSignal::Load(SceneType::MapGenTest(entry))
        } else {
            SceneSignal::None
        }
    }
}

impl MapGenSelectionScene {
    pub fn new() -> MapGenSelectionScene {
        let mut entries: Vec<MapGenEntry> = Vec::new();
        entries.push(MapGenEntry {
            map_gen_type: MapGenType::Rooms,
            label: "Rooms and corridor",
        });
        entries.push(MapGenEntry {
            map_gen_type: MapGenType::Buildings,
            label: "Buildings",
        });
        MapGenSelectionScene {
            entries,
        }
    }

    fn read_input(&mut self, ctx: &BTerm) -> Option<MapGenType> {
        let options = self
            .entries
            .iter()
            .map(|entry| (entry.label.to_string(), entry.map_gen_type))
            .collect();
        match read_input_selection(ctx.key, &options) {
            ItemMenuResult::Selected { result } => Some(result),
            _ => None,
        }
    }

    fn draw(&self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(
            15,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            "Rust Roguelike Tutorial",
        );

        for (i, entry) in self.entries.iter().enumerate() {
            ctx.print_color_centered(
                24 + i,
                RGB::named(WHITE),
                RGB::named(BLACK),
                format_option(i, entry.label),
            );
        }
    }
}
