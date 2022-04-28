use bracket_lib::prelude::*;
use specs::{World, WorldExt};

use crate::{
    game_display::UiSignal,
    input::{get_direction_offset, map_all, map_direction, map_look_commands, Command},
    resources::PointsOfInterest,
};

use super::{
    game_ui::draw_tooltips,
    gui_handlers::{ItemMenuResult, UiHandler, UiScreen},
};

#[derive(PartialEq, Copy, Clone)]
pub struct ExamineHandler {
    pub selection: Point,
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
