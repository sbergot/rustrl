use bracket_lib::prelude::*;
use specs::prelude::*;
use specs::saveload::*;

use crate::components::*;
use crate::gamelog::GameLog;
use crate::gui::game_ui::draw_ui;
use crate::gui::gui_handlers::*;
use crate::gui::main_menu::*;
use crate::map;
use crate::map::Map;
use crate::player;
use crate::player::PlayerEntity;
use crate::player::PlayerPos;
use crate::spawner;
use crate::systems::*;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowUi { screen: UiScreen },
    MainMenu { menu_selection : MainMenuSelection },
    SaveGame,
}

pub struct State<'a, 'b> {
    pub ecs: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> State<'a, 'b> {
    fn run_systems(&mut self) {
        self.dispatcher.dispatch(&self.ecs);
        self.ecs.maintain();
    }

    fn draw_renderables(&mut self, ctx: &mut BTerm) {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render) in data {
            let idx = map.xy_idx(pos.pos);
            if map.visible_tiles[idx] {
                ctx.set(pos.pos.x, pos.pos.y, render.fg, render.bg, render.glyph)
            }
        }
    }
}

impl GameState for State<'static, 'static> {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::MainMenu {..} => {}
            _ => {
                Map::draw_map(&self.ecs, ctx);
                self.draw_renderables(ctx);
            }
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player::player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowUi { screen } => {
                let res = run_screen(&mut self.ecs, ctx, screen);
                if let Some(newstate) = res {
                    newrunstate = newstate;
                }
            },
            RunState::MainMenu {..} => {
                let result = main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
                    MainMenuResult::Selected{ selected } => {
                        match selected {
                            MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            MainMenuSelection::LoadGame => {
                                load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                            }
                            MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            },
            RunState::SaveGame => {
                save_game(&mut self.ecs);

                newrunstate = RunState::MainMenu{ menu_selection : MainMenuSelection::LoadGame };
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        draw_ui(&self.ecs, ctx);
    }
}

pub fn init_state<'a, 'b>(width: i32, height: i32) -> State<'a, 'b> {
    let mut world = World::new();

    let mut dispatcher = with_systems(DispatcherBuilder::new()).build();
    dispatcher.setup(&mut world);
    world.register::<Renderable>();
    world.register::<Item>();
    world.register::<ProvidesHealing>();
    world.register::<Ranged>();
    world.register::<InflictsDamage>();

    world.register::<SimpleMarker<SerializeMe>>();
    world.register::<SerializationHelper>();
    world.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let mut gs = State {
        ecs: world,
        dispatcher,
    };

    let (rooms, map) = map::Map::new_map_rooms_and_corridors(width, height);
    gs.ecs.insert(map);
    gs.ecs.insert(RunState::MainMenu { menu_selection: MainMenuSelection::NewGame });
    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });
    gs.ecs.insert(RandomNumberGenerator::new());

    let room_center = rooms[0].center();
    let player_entity = spawner::player(&mut gs.ecs, room_center);

    gs.ecs.insert(PlayerEntity {
        entity: player_entity,
    });

    gs.ecs.insert(PlayerPos { pos: room_center });

    for room in rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs
}
