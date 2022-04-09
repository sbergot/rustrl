use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::components::*;
use crate::gamelog::GameLog;
use crate::gui;
use crate::gui_handlers::UiScreen;
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

    fn use_selected_item(&mut self, item_entity: Entity) -> RunState {
        let is_ranged = self.ecs.read_storage::<Ranged>();
        let is_item_ranged = is_ranged.get(item_entity);
        if let Some(is_item_ranged) = is_item_ranged {
            RunState::ShowUi {
                screen: UiScreen::Targeting {
                    range: is_item_ranged.range,
                    item: item_entity,
                },
            }
        } else {
            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
            intent
                .insert(
                    self.ecs.fetch::<PlayerEntity>().entity,
                    WantsToUseItem {
                        item: item_entity,
                        target: None,
                    },
                )
                .expect("Unable to insert intent");
            RunState::PlayerTurn
        }
    }
}

impl GameState for State<'static, 'static> {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        Map::draw_map(&self.ecs, ctx);

        {
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
            RunState::ShowUi { screen } => match screen {
                UiScreen::Inventory => {
                    let result = gui::show_inventory(&mut self.ecs, ctx);
                    match result.0 {
                        gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                        gui::ItemMenuResult::NoResponse => {}
                        gui::ItemMenuResult::Selected => {
                            let item_entity = result.1.unwrap();
                            newrunstate = self.use_selected_item(item_entity);
                        }
                    }
                }
                UiScreen::DropItem => {
                    let result = gui::drop_item_menu(&mut self.ecs, ctx);
                    match result.0 {
                        gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                        gui::ItemMenuResult::NoResponse => {}
                        gui::ItemMenuResult::Selected => {
                            let item_entity = result.1.unwrap();
                            let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                            intent
                                .insert(
                                    self.ecs.fetch::<PlayerEntity>().entity,
                                    WantsToDropItem { item: item_entity },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
                UiScreen::Targeting { range, item } => {
                    let result = gui::ranged_target(&mut self.ecs, ctx, range);
                    match result.0 {
                        gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                        gui::ItemMenuResult::NoResponse => {}
                        gui::ItemMenuResult::Selected => {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    self.ecs.fetch::<PlayerEntity>().entity,
                                    WantsToUseItem {
                                        item,
                                        target: result.1,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
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
    world.register::<ProvidesHealing>();
    world.register::<Ranged>();
    world.register::<InflictsDamage>();

    let mut gs = State {
        ecs: world,
        dispatcher,
    };

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
