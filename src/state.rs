use bracket_lib::prelude::*;
use specs::{prelude::*, saveload::*};

use crate::{
    components::*,
    game_display::{GameDisplay, GameSignal},
    gamelog::GameLog,
    map_generation,
    resources::*,
    scenes::{Scene, SceneSignal, SceneType},
    spawner,
    systems::*,
};

pub struct State<'a, 'b> {
    ecs: World,
    gameplay_systems: Dispatcher<'a, 'b>,
    indexing_systems: Dispatcher<'a, 'b>,
    runstate: RunState,
    display: GameDisplay,
}

impl<'a, 'b> State<'a, 'b> {
    fn run_systems(&mut self) {
        self.gameplay_systems.dispatch(&self.ecs);
        self.ecs.maintain();
        self.indexing_systems.dispatch(&self.ecs);
        self.ecs.maintain();
    }

    fn is_player_dead(&mut self) -> bool {
        let entities = self.ecs.entities();
        let player_entity = self.ecs.read_resource::<PlayerEntity>().entity;
        !entities.is_alive(player_entity)
    }

    pub fn load_game(&mut self) {
        load_game(&mut self.ecs);
    }
}

impl<'a, 'b> Scene for State<'a, 'b> {
    fn tick(&mut self, ctx: &mut BTerm) -> SceneSignal {
        if self.is_player_dead() {
            return SceneSignal::Load(SceneType::GameOver);
        }

        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        self.display.draw(&self.ecs, ctx);

        match self.runstate {
            RunState::PreRun => {
                self.run_systems();
                self.runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                self.run_systems();
                match self.display.read_input(&self.ecs, ctx) {
                    GameSignal::None => {}
                    GameSignal::Perform(action) => {
                        let player_entity = self.ecs.read_resource::<PlayerEntity>().entity;
                        action.run(player_entity, &mut self.ecs);
                        self.runstate = RunState::PlayerTurn;
                    }
                    GameSignal::SaveQuit => {
                        save_game(&mut self.ecs);
                        return SceneSignal::Load(SceneType::MainMenu);
                    }
                }
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                run_monster_ai(&mut self.ecs);
                self.runstate = RunState::AwaitingInput;
            }
        }

        SceneSignal::None
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
        runstate: RunState::PreRun,
        display: GameDisplay::new(),
    };

    gs.ecs.insert(RandomNumberGenerator::new());

    let mut generator =
        map_generation::buildings_generator::BuildingsGenerator::new(width, height);
    let (rooms, map) = generator.new_map_buildings();

    let room_center = rooms[0].center();
    let player_entity = spawner::player(&mut gs.ecs, room_center);

    for room in rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    gs.ecs.insert(PlayerEntity {
        entity: player_entity,
    });

    gs.ecs.insert(PlayerPos { pos: room_center });

    gs.ecs.insert(PointsOfInterest::new());

    gs
}
