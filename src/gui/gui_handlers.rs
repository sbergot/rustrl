use bracket_lib::prelude::*;
use specs::*;

use crate::{
    actions::*,
    components::*,
    game_display::{UiSignal, try_move_player, GameSignal},
    gui::{components::*, game_ui::*},
    input::*,
    queries::*,
    resources::{PlayerPos, PointsOfInterest}, player::grab_item,
};

#[derive(PartialEq, Copy, Clone)]
pub enum ItemUsage {
    Use,
    Drop,
    Equip,
    Unequip,
}

#[derive(PartialEq, Copy, Clone)]
pub enum UiScreen {
    Inventory,
    UseItem {
        item: Entity,
    },
    RemoveItem,
    Targeting {
        range: i32,
        item: Entity,
        selection: Point,
    },
    Examine {
        selection: Point,
    },
    Play,
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult<T> {
    Cancel,
    NoResponse,
    Selected { result: T },
}

pub fn read_input_selection<T: Copy>(
    key: Option<VirtualKeyCode>,
    options: &Vec<(String, T)>,
) -> ItemMenuResult<T> {
    let count = options.len();

    match key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return ItemMenuResult::Selected {
                        result: options[selection as usize].1,
                    };
                }
                ItemMenuResult::NoResponse
            }
        },
    }
}

pub fn get_screen_handler(screen: UiScreen) -> Box<dyn UiHandlerMin> {
    match screen {
        UiScreen::Play => Box::new(PlayHandler {}),
        UiScreen::Inventory => Box::new(InventoryHandler {}),
        UiScreen::Targeting {
            range,
            item,
            selection,
        } => Box::new(TargetingHandler {
            range,
            item,
            selection,
        }),
        UiScreen::RemoveItem => Box::new(EquippedItemHandler {}),
        UiScreen::Examine { selection } => Box::new(ExamineHandler { selection }),
        UiScreen::UseItem { item } => Box::new(UseItemHandler { item }),
    }
}

pub fn draw_screen(ecs: &World, ctx: &mut BTerm, screen: UiScreen) {
    get_screen_handler(screen).show(ecs, ctx);
}

pub fn run_screen(ecs: &World, ctx: &mut BTerm, screen: UiScreen) -> UiSignal {
    get_screen_handler(screen).run_handler(ecs, ctx)
}

trait UiHandler {
    type Output;

    fn show(&self, ecs: &World, ctx: &mut BTerm);

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output>;

    fn handle(&self, ecs: &World, input: Self::Output) -> UiSignal;

    fn run_handler(&self, ecs: &World, ctx: &mut BTerm) -> UiSignal {
        self.show(ecs, ctx);
        let menuresult = self.read_input(ecs, ctx);
        match menuresult {
            ItemMenuResult::Cancel => UiSignal::UpdateScreen(UiScreen::Play),
            ItemMenuResult::NoResponse => UiSignal::None,
            ItemMenuResult::Selected { result } => self.handle(ecs, result),
        }
    }
}

pub trait UiHandlerMin {
    fn show(&self, ecs: &World, ctx: &mut BTerm);
    fn run_handler(&self, ecs: &World, ctx: &mut BTerm) -> UiSignal;
}

impl<T> UiHandlerMin for T where T: UiHandler,
{
    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        self.show(ecs, ctx);
    }

    fn run_handler(&self, ecs: &World, ctx: &mut BTerm) -> UiSignal {
        self.run_handler(ecs, ctx)
    }
}


#[derive(PartialEq, Copy, Clone)]
struct PlayHandler {}

impl UiHandler for PlayHandler {
    type Output = Command;

    fn show(&self, _ecs: &World, _ctx: &mut BTerm) {}

    fn read_input(&self, _ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let cmd = map_all(ctx.key, &[map_direction, map_other_commands]);
        match cmd {
            Some(cmd) => ItemMenuResult::Selected { result: cmd },
            None => ItemMenuResult::NoResponse,
        }
    }

    fn handle(&self, ecs: &World, input: Command) -> UiSignal {
        match input {
            Command::Direction { direction } => try_move_player(direction, ecs),
            Command::Wait => UiSignal::GameSignal(GameSignal::Perform(Box::new(WaitAction {}))),
            Command::Grab => {
                let action = grab_item(ecs);
                if let Some(action) = action {
                    UiSignal::GameSignal(GameSignal::Perform(action))
                } else {
                    UiSignal::None
                }
            }
            Command::ShowInventory => return UiSignal::UpdateScreen(UiScreen::Inventory),
            Command::ShowRemoveItem => return UiSignal::UpdateScreen(UiScreen::RemoveItem),
            Command::ExamineMode => {
                let player_pos = ecs.read_resource::<PlayerPos>();
                return UiSignal::UpdateScreen(UiScreen::Examine {
                    selection: player_pos.pos,
                });
            }
            Command::SaveQuit => return UiSignal::GameSignal(GameSignal::SaveQuit),
            _ => UiSignal::None,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct InventoryHandler {}

impl UiHandler for InventoryHandler {
    type Output = Entity;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        let options = get_inventory_options(ecs);
        show_selection(ctx, "Inventory", &options);
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_inventory_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, _ecs: &World, input: Entity) -> UiSignal {
        UiSignal::UpdateScreen(UiScreen::UseItem { item: input })
    }
}

fn try_use_item(ecs: &World, input: Entity) -> UiSignal {
    {
        let is_ranged = ecs.read_storage::<Ranged>();
        let is_item_ranged = is_ranged.get(input);
        if let Some(is_item_ranged) = is_item_ranged {
            let player_pos = ecs.read_resource::<PlayerPos>();
            return UiSignal::UpdateScreen(UiScreen::Targeting {
                range: is_item_ranged.range,
                item: input,
                selection: player_pos.pos,
            });
        }
    }
    let action = UseItemAction {
        item: input,
        target: None,
    };
    UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
}

#[derive(PartialEq, Copy, Clone)]
struct TargetingHandler {
    range: i32,
    item: Entity,
    selection: Point,
}

enum LookCommand {
    Inspect(Point),
    Select(Point),
}

impl UiHandler for TargetingHandler {
    type Output = LookCommand;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        ctx.print_color(
            5,
            0,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            "Select Target:",
        );

        let available_cells = get_cells_in_range(ecs, self.range);

        for tile in available_cells.iter() {
            ctx.set_bg(tile.x, tile.y, RGB::named(BLUE));
        }

        let pos = self.selection;
        let color = if available_cells.contains(&pos) {
            RGB::named(CYAN)
        } else {
            RGB::named(RED)
        };
        ctx.set_bg(pos.x, pos.y, color);
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let input = map_all(ctx.key, &[map_direction, map_look_commands]);
        match input {
            None => ItemMenuResult::NoResponse,
            Some(cmd) => match cmd {
                Command::Direction { direction } => ItemMenuResult::Selected {
                    result: LookCommand::Inspect(self.selection + get_direction_offset(direction)),
                },
                Command::NextTarget => {
                    let poi = ecs.read_resource::<PointsOfInterest>();
                    let next_pos = poi.get_next(self.selection);
                    match next_pos {
                        Some(next_pos) => ItemMenuResult::Selected {
                            result: LookCommand::Inspect(next_pos),
                        },
                        None => ItemMenuResult::NoResponse,
                    }
                }
                Command::Validate => ItemMenuResult::Selected {
                    result: LookCommand::Select(self.selection),
                },
                Command::Cancel => ItemMenuResult::Cancel,
                _ => ItemMenuResult::NoResponse,
            },
        }
    }

    fn handle(&self, _ecs: &World, input: LookCommand) -> UiSignal {
        match input {
            LookCommand::Select(selection) => {
                let action = UseItemAction {
                    item: self.item,
                    target: Some(selection),
                };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            LookCommand::Inspect(point) => UiSignal::UpdateScreen(UiScreen::Targeting {
                range: self.range,
                item: self.item,
                selection: point,
            }),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
struct EquippedItemHandler {}

impl UiHandler for EquippedItemHandler {
    type Output = Entity;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        let options = get_equipped_options(ecs);
        show_selection(ctx, "Remove Which Item?", &options)
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_equipped_options(ecs);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, _ecs: &World, input: Entity) -> UiSignal {
        UiSignal::UpdateScreen(UiScreen::UseItem { item: input })
    }
}

#[derive(PartialEq, Copy, Clone)]
struct ExamineHandler {
    selection: Point,
}

impl UiHandler for ExamineHandler {
    type Output = Point;

    fn show(&self, _ecs: &World, ctx: &mut BTerm) {
        ctx.print_color(5, 0, RGB::named(YELLOW), RGB::named(BLACK), "Examine mode");

        let pos = self.selection;
        ctx.set_bg(pos.x, pos.y, RGB::named(CYAN));
        draw_tooltips(_ecs, ctx, pos);
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let input = map_all(ctx.key, &[map_direction, map_look_commands]);
        match input {
            None => ItemMenuResult::NoResponse,
            Some(cmd) => match cmd {
                Command::Direction { direction } => ItemMenuResult::Selected {
                    result: self.selection + get_direction_offset(direction),
                },
                Command::NextTarget => {
                    let poi = ecs.read_resource::<PointsOfInterest>();
                    let next_pos = poi.get_next(self.selection);
                    match next_pos {
                        Some(next_pos) => ItemMenuResult::Selected { result: next_pos },
                        None => ItemMenuResult::NoResponse,
                    }
                }
                Command::Validate => ItemMenuResult::NoResponse,
                Command::Cancel => ItemMenuResult::Cancel,
                _ => ItemMenuResult::NoResponse,
            },
        }
    }

    fn handle(&self, _ecs: &World, input: Point) -> UiSignal {
        UiSignal::UpdateScreen(UiScreen::Examine { selection: input })
    }
}

#[derive(PartialEq, Copy, Clone)]
struct UseItemHandler {
    item: Entity,
}

impl UiHandler for UseItemHandler {
    type Output = ItemUsage;

    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        let options = get_usage_options(ecs, self.item);
        show_selection(ctx, "Pick action", &options);
    }

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output> {
        let options = get_usage_options(ecs, self.item);
        read_input_selection(ctx.key, &options)
    }

    fn handle(&self, ecs: &World, input: ItemUsage) -> UiSignal {
        match input {
            ItemUsage::Drop => {
                let action = DropItemAction { target: self.item };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            ItemUsage::Equip => {
                let action = EquipItemAction { target: self.item };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            ItemUsage::Unequip => {
                let action = UnequipItemAction { target: self.item };
                UiSignal::GameSignal(GameSignal::Perform(Box::new(action)))
            }
            ItemUsage::Use => {
                return try_use_item(ecs, self.item);
            }
        }
    }
}
