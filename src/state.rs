use bracket_lib::prelude::*;
use specs::{prelude::*, saveload::*};

use crate::{
    components::*,
    game_map::GameMap,
    gamelog::GameLog,
    gui::{game_ui::*, gui_handlers::*, main_menu::*},
    map::Map,
    map_generation, player,
    resources::{PlayerEntity, PlayerPos, PointsOfInterest, RunState},
    spawner,
    systems::*,
};

pub struct State<'a, 'b> {
    pub ecs: World,
    gameplay_systems: Dispatcher<'a, 'b>,
    indexing_systems: Dispatcher<'a, 'b>,
}

impl<'a, 'b> State<'a, 'b> {
    fn run_systems(&mut self) {
        self.gameplay_systems.dispatch(&self.ecs);
        self.ecs.maintain();
        self.indexing_systems.dispatch(&self.ecs);
        self.ecs.maintain();
    }

    fn draw_renderables(&mut self, ctx: &mut BTerm) {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.read_resource::<GameMap>();
        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render) in data {
            let idx = map.xy_idx(pos.pos);
            if map.visible_tiles[idx] {
                ctx.set(pos.pos.x, pos.pos.y, render.fg, render.bg, render.glyph)
            }
        }
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    fn is_player_dead(&mut self) -> bool {
        let entities = self.ecs.entities();
        let player_entity = self.ecs.read_resource::<PlayerEntity>().entity;
        !entities.is_alive(player_entity)
    }
}

impl GameState for State<'static, 'static> {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        let mut newrunstate;
        {
            let runstate = self.ecs.read_resource::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::MainMenu { .. } => {}
            _ => {
                GameMap::draw_map(&self.ecs, ctx);
                self.draw_renderables(ctx);
            }
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                self.run_systems();

                if self.is_player_dead() {
                    newrunstate = RunState::GameOver;
                } else {
                    newrunstate = player::player_input(&mut self.ecs, ctx.key);
                }
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                run_monster_ai(&mut self.ecs);
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowUi { screen } => {
                let res = run_screen(&mut self.ecs, ctx, screen);
                if let Some(newstate) = res {
                    newrunstate = newstate;
                }
            }
            RunState::MainMenu { .. } => {
                let result = main_menu(newrunstate, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    MainMenuResult::Selected { selected } => match selected {
                        MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        MainMenuSelection::LoadGame => {
                            load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                        }
                        MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                save_game(&mut self.ecs);

                newrunstate = RunState::MainMenu {
                    menu_selection: MainMenuSelection::LoadGame,
                };
            }
            RunState::GameOver => {
                let result = game_over(ctx);
                match result {
                    GameOverResult::NoSelection => {}
                    GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu {
                            menu_selection: MainMenuSelection::NewGame,
                        };
                    }
                }
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

    let mut gameplay_dispatcher = with_gameplay_systems(DispatcherBuilder::new()).build();
    gameplay_dispatcher.setup(&mut world);

    let mut indexing_dispatcher = with_indexing_systems(DispatcherBuilder::new()).build();
    indexing_dispatcher.setup(&mut world);

    world.register::<Renderable>();
    world.register::<Item>();
    world.register::<ProvidesHealing>();
    world.register::<Ranged>();
    world.register::<InflictsDamage>();
    world.register::<Consumable>();
    world.register::<Monster>();
    world.register::<Equippable>();
    world.register::<MeleePowerBonus>();
    world.register::<AreaOfEffect>();
    world.register::<DefenseBonus>();
    world.register::<Confused>();
    world.register::<Confusion>();
    world.register::<Inventory>();
    world.register::<Equipment>();

    world.register::<SimpleMarker<SerializeMe>>();
    world.register::<SerializationHelper>();
    world.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    world.insert(particle_system::ParticleBuilder::new());

    let mut gs = State {
        ecs: world,
        gameplay_systems: gameplay_dispatcher,
        indexing_systems: indexing_dispatcher,
    };

    let mut generator =
        map_generation::rooms_corridors::RoomsCorridorsGenerator::new(width, height);
    let (rooms, map) = generator.new_map_rooms_and_corridors();

    gs.ecs.insert(map);
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: MainMenuSelection::NewGame,
    });
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

    gs.ecs.insert(PointsOfInterest::new());

    for room in rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs
}
