use bracket_lib::prelude::*;

use crate::{
    gui::components::format_option,
    input::{read_input_selection, ItemMenuResult},
    scenes::{Scene, SceneSignal, SceneType},
    systems::does_save_exist,
};

#[derive(Clone)]
struct MainMenuEntry {
    scene: SceneType,
    label: &'static str,
}

pub struct MainMenuScene {
    entries: Vec<MainMenuEntry>,
}

impl Scene for MainMenuScene {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        self.draw(ctx);
        if let Some(entry) = self.read_input(ctx) {
            SceneSignal::Load(entry)
        } else {
            SceneSignal::None
        }
    }
}

impl MainMenuScene {
    pub fn new() -> MainMenuScene {
        let mut entries: Vec<MainMenuEntry> = Vec::new();
        entries.push(MainMenuEntry {
            scene: SceneType::NewGame,
            label: "Begin New Game",
        });
        if does_save_exist() {
            entries.push(MainMenuEntry {
                scene: SceneType::LoadGame,
                label: "Load Game",
            });
        }
        entries.push(MainMenuEntry {
            scene: SceneType::MapGenSelection,
            label: "Test map gen",
        });
        entries.push(MainMenuEntry {
            scene: SceneType::Quit,
            label: "Quit",
        });
        MainMenuScene {
            entries,
        }
    }

    fn read_input(&mut self, ctx: &BTerm) -> Option<SceneType> {
        let options = self
            .entries
            .iter()
            .map(|entry| (entry.label.to_string(), entry.scene))
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
