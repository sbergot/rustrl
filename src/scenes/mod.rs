mod game_over_scene;
mod game_scene;
mod main_menu_scene;

use bracket_lib::prelude::GameState;

use self::{game_over_scene::GameOverScene, game_scene::GameScene, main_menu_scene::MainMenuScene};

#[derive(Clone, Copy, PartialEq)]
pub enum SceneType {
    MainMenu,
    NewGame,
    LoadGame,
    GameOver,
    Quit,
}

pub enum SceneSignal {
    Load(SceneType),
    None,
}

pub trait Scene {
    fn tick(&mut self, ctx: &mut bracket_lib::prelude::BTerm) -> SceneSignal;
}

type AnyScene = Box<dyn Scene>;

pub struct SceneHandler {
    current_scene: AnyScene,
}

impl SceneHandler {
    pub fn new() -> SceneHandler {
        SceneHandler {
            current_scene: Box::new(MainMenuScene::new()),
        }
    }
}

impl GameState for SceneHandler {
    fn tick(&mut self, ctx: &mut bracket_lib::prelude::BTerm) {
        match self.current_scene.tick(ctx) {
            SceneSignal::Load(scene) => {
                self.current_scene = load_scene(scene);
            }
            SceneSignal::None => {}
        }
    }
}

fn load_scene(scene: SceneType) -> AnyScene {
    match scene {
        SceneType::MainMenu => Box::new(MainMenuScene::new()),
        SceneType::NewGame => Box::new(GameScene::new_game()),
        SceneType::LoadGame => Box::new(GameScene::load_game()),
        SceneType::GameOver => Box::new(GameOverScene {}),
        SceneType::Quit => {
            ::std::process::exit(0);
        }
    }
}
