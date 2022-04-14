use bracket_lib::prelude::*;

use crate::state::*;
use crate::systems::does_save_exist;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs : &mut State, ctx : &mut BTerm) -> MainMenuResult {
    let save_exists = does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(YELLOW), RGB::named(BLACK), "Rust Roguelike Tutorial");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(MAGENTA), RGB::named(BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(WHITE), RGB::named(BLACK), "Begin New Game");
        }

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(25, RGB::named(MAGENTA), RGB::named(BLACK), "Load Game");
            } else {
                ctx.print_color_centered(25, RGB::named(WHITE), RGB::named(BLACK), "Load Game");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(MAGENTA), RGB::named(BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(WHITE), RGB::named(BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}