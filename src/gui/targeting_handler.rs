use bracket_lib::prelude::*;
use specs::{Entity, World, WorldExt};

use crate::{
    actions::UseItemAction,
    components::Viewshed,
    game_display::{GameSignal, UiSignal},
    input::{get_direction_offset, map_all, map_direction, map_look_commands, Command, ItemMenuResult},
    resources::{PlayerEntity, PlayerPos, PointsOfInterest},
};

use super::gui_handlers::{UiHandler, UiScreen};

#[derive(PartialEq, Copy, Clone)]
pub struct TargetingHandler {
    pub range: i32,
    pub item: Entity,
    pub selection: Point,
}

pub enum LookCommand {
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

pub fn get_cells_in_range(ecs: &World, range: i32) -> Vec<Point> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let player_pos = ecs.read_resource::<PlayerPos>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(player_entity.entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for tile in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(player_pos.pos, *tile);
            if distance <= range as f32 {
                available_cells.push(*tile);
            }
        }
    }
    available_cells
}
