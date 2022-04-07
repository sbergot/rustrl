use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::systems::*;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::gui;
use crate::map;
use crate::map::Map;
use crate::player;
use crate::player::PlayerEntity;
use crate::player::PlayerPos;
use crate::spawner;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State<'a, 'b> {
    pub ecs: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> State<'a, 'b> {
    fn run_systems(&mut self) {
        self.dispatcher.dispatch(&self.ecs);
        delete_the_dead(&mut self.ecs);
        self.ecs.maintain();
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
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        Map::draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.pos);
            if map.visible_tiles[idx] {
                ctx.set(pos.pos.x, pos.pos.y, render.fg, render.bg, render.glyph)
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}

pub fn init_state<'a, 'b>(width: i32, height: i32) -> State<'a, 'b> {
    let mut world = World::new();

    let mut dispatcher = with_systems(DispatcherBuilder::new()).build();
    dispatcher.setup(&mut world);
    world.register::<Renderable>();
    world.register::<Item>();
    world.register::<Potion>();

    let mut gs = State { ecs: world, dispatcher };

    let (rooms, map) = map::Map::new_map_rooms_and_corridors(width, height);
    gs.ecs.insert(map);
    gs.ecs.insert(RunState::PreRun);
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
