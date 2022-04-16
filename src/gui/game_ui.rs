use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{
    components::*,
    constants::UI_HEIGHT,
    gamelog::GameLog,
    map::Map,
    player::{PlayerEntity, PlayerPos},
};

use super::components::show_selection;

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.read_resource::<Map>();
    ctx.draw_box(
        0,
        map.height,
        map.width - 1,
        UI_HEIGHT - 1,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            map.height,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            &health,
        );

        ctx.draw_bar_horizontal(
            28,
            map.height,
            map.width - 28 - 1,
            stats.hp,
            stats.max_hp,
            RGB::named(RED),
            RGB::named(BLACK),
        );
    }

    let log = ecs.fetch::<GameLog>();

    let mut y = map.height + 1;
    let max_y = map.height + UI_HEIGHT - 2;
    for s in log.entries.iter().rev() {
        if y < map.width - 1 {
            ctx.print(2, y, s);
        }
        y += 1;
        if y > max_y {
            break;
        }
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.pos);
        if position.pos.x == mouse_pos.0 && position.pos.y == mouse_pos.1 && map.visible_tiles[idx]
        {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = ecs.fetch::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let options: Vec<(&str, Entity)> = (&backpack, &names, &entities)
        .join()
        .filter(|(item, _name, _e)| item.owner == player_entity.entity)
        .map(|(_i, name, entity)| (name.name.as_str(), entity))
        .collect();

    show_selection(ctx, "Inventory", &options)
}

pub fn drop_item_menu(ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = ecs.fetch::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let options: Vec<(&str, Entity)> = (&backpack, &names, &entities)
        .join()
        .filter(|(item, _name, _e)| item.owner == player_entity.entity)
        .map(|(_i, name, entity)| (name.name.as_str(), entity))
        .collect();

    show_selection(ctx, "Drop Which Item?", &options)
}

pub fn remove_item_menu(ecs: &mut World, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = ecs.fetch::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<Equipped>();
    let entities = ecs.entities();

    let options: Vec<(&str, Entity)> = (&entities, &backpack, &names)
        .join()
        .filter(|(_entity, pack, _name)| pack.owner == player_entity.entity)
        .map(|(entity, _pack, name)| (name.name.as_str(), entity))
        .collect();

    show_selection(ctx, "Remove Which Item?", &options)
}

pub fn ranged_target(
    ecs: &mut World,
    ctx: &mut BTerm,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let player_entity = ecs.fetch::<PlayerEntity>();
    let player_pos = ecs.fetch::<PlayerPos>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Select Target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(player_entity.entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(player_pos.pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_point();
    let valid_target = available_cells.iter().any(|c| **c == mouse_pos);

    if valid_target {
        ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(mouse_pos));
        }
    } else {
        ctx.set_bg(mouse_pos.x, mouse_pos.y, RGB::named(RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}
