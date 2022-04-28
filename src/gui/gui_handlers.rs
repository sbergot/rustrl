use bracket_lib::prelude::*;
use specs::*;

use crate::game_display::UiSignal;

use super::{
    equipped_item_handler::EquippedItemHandler, examine_handler::ExamineHandler,
    inventory_handler::InventoryHandler, play_handler::PlayHandler,
    targeting_handler::TargetingHandler, use_item_handler::UseItemHandler,
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

fn get_screen_handler(screen: UiScreen) -> Box<dyn UiHandlerMin> {
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

pub trait UiHandler {
    type Output;

    fn show(&self, ecs: &World, ctx: &mut BTerm);

    fn read_input(&self, ecs: &World, ctx: &mut BTerm) -> ItemMenuResult<Self::Output>;

    fn handle(&self, ecs: &World, input: Self::Output) -> UiSignal;

    fn run_handler(&self, ecs: &World, ctx: &mut BTerm) -> UiSignal {
        let menuresult = self.read_input(ecs, ctx);
        match menuresult {
            ItemMenuResult::Cancel => UiSignal::UpdateScreen(UiScreen::Play),
            ItemMenuResult::NoResponse => UiSignal::None,
            ItemMenuResult::Selected { result } => self.handle(ecs, result),
        }
    }
}

trait UiHandlerMin {
    fn show(&self, ecs: &World, ctx: &mut BTerm);
    fn run_handler(&self, ecs: &World, ctx: &mut BTerm) -> UiSignal;
}

impl<T> UiHandlerMin for T
where
    T: UiHandler,
{
    fn show(&self, ecs: &World, ctx: &mut BTerm) {
        self.show(ecs, ctx);
    }

    fn run_handler(&self, ecs: &World, ctx: &mut BTerm) -> UiSignal {
        self.run_handler(ecs, ctx)
    }
}
