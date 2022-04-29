use bracket_lib::prelude::*;

use crate::{
    input::{map_all, map_direction, map_look_commands, Command, Direction},
    scenes::{Scene, SceneSignal, SceneType},
    systems::does_save_exist,
};

#[derive(Clone)]
struct MainMenuEntry {
    scene: SceneType,
    label: &'static str,
}

pub struct MainMenuScene {
    selection: i32,
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
            scene: SceneType::Quit,
            label: "Quit",
        });
        MainMenuScene {
            selection: 0,
            entries,
        }
    }

    fn read_input(&mut self, ctx: &BTerm) -> Option<SceneType> {
        let input = map_all(ctx.key, &[map_direction, map_look_commands]);

        match input {
            None => None,
            Some(key) => match key {
                Command::Direction {
                    direction: Direction::Up,
                } => {
                    self.selection = (self.selection - 1).rem_euclid(self.entries.len() as i32);
                    None
                }
                Command::Direction {
                    direction: Direction::Down,
                } => {
                    self.selection = (self.selection + 1).rem_euclid(self.entries.len() as i32);
                    None
                }
                Command::Validate => Some(self.entries[self.selection as usize].scene),
                _ => None,
            },
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

        ctx.print_color_centered(23, RGB::named(WHITE), RGB::named(BLACK), self.selection);

        for (i, entry) in self.entries.iter().enumerate() {
            let color = if i == self.selection as usize {
                RGB::named(MAGENTA)
            } else {
                RGB::named(WHITE)
            };
            ctx.print_color_centered(24 + i, color, RGB::named(BLACK), entry.label);
        }
    }
}
