use bracket_lib::prelude::*;

use crate::{
    scenes::{Scene, SceneSignal, SceneType},
    systems::does_save_exist,
};

pub struct MainMenuScene {
    selection: MainMenuSelection,
    save_exists: bool,
}

impl Scene for MainMenuScene {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        self.draw(ctx);
        match self.read_input(ctx) {
            MainMenuResult::NoSelection { selected } => {
                self.selection = selected;
                SceneSignal::None
            }
            MainMenuResult::Selected { selected } => {
                let new_scene = match selected {
                    MainMenuSelection::NewGame => SceneType::NewGame,
                    MainMenuSelection::LoadGame => SceneType::LoadGame,
                    MainMenuSelection::Quit => SceneType::Quit,
                };
                SceneSignal::Load(new_scene)
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

impl MainMenuScene {
    pub fn new() -> MainMenuScene {
        MainMenuScene {
            selection: MainMenuSelection::NewGame,
            save_exists: does_save_exist(),
        }
    }

    fn read_input(&self, ctx: &BTerm) -> MainMenuResult {
        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: self.selection,
                }
            }
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    let mut newselection;
                    match self.selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !self.save_exists {
                        newselection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Down => {
                    let mut newselection;
                    match self.selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !self.save_exists {
                        newselection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: self.selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: self.selection,
                    }
                }
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

        {
            if self.selection == MainMenuSelection::NewGame {
                ctx.print_color_centered(
                    24,
                    RGB::named(MAGENTA),
                    RGB::named(BLACK),
                    "Begin New Game",
                );
            } else {
                ctx.print_color_centered(
                    24,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    "Begin New Game",
                );
            }

            if self.save_exists {
                if self.selection == MainMenuSelection::LoadGame {
                    ctx.print_color_centered(
                        25,
                        RGB::named(MAGENTA),
                        RGB::named(BLACK),
                        "Load Game",
                    );
                } else {
                    ctx.print_color_centered(25, RGB::named(WHITE), RGB::named(BLACK), "Load Game");
                }
            }

            if self.selection == MainMenuSelection::Quit {
                ctx.print_color_centered(26, RGB::named(MAGENTA), RGB::named(BLACK), "Quit");
            } else {
                ctx.print_color_centered(26, RGB::named(WHITE), RGB::named(BLACK), "Quit");
            }
        }
    }
}
