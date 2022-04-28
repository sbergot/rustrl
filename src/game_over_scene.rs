use bracket_lib::prelude::*;

use crate::scenes::{Scene, SceneSignal, SceneType};

pub struct GameOverScene;

impl Scene for GameOverScene {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        draw(ctx);
        match read_input(ctx) {
            GameOverResult::NoSelection => SceneSignal::None,
            GameOverResult::QuitToMenu => SceneSignal::Load(SceneType::MainMenu)
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

fn read_input(ctx: &BTerm) -> GameOverResult {
    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}

fn draw(ctx: &mut BTerm) {
    ctx.cls();
    ctx.print_color_centered(
        15,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Your journey has ended!",
    );
    ctx.print_color_centered(
        17,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "One day, we'll tell you all about how you did.",
    );
    ctx.print_color_centered(
        18,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "That day, sadly, is not in this chapter..",
    );

    ctx.print_color_centered(
        20,
        RGB::named(MAGENTA),
        RGB::named(BLACK),
        "Press any key to return to the menu.",
    );
}
