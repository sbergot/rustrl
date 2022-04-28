use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{components::*, constants::UI_HEIGHT, game_map::GameMap, gamelog::GameLog, map::Map};

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.read_resource::<GameMap>();
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

    let log = ecs.read_resource::<GameLog>();

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
}

pub fn draw_tooltips(ecs: &World, ctx: &mut BTerm, pos: Point) {
    let map = ecs.read_resource::<GameMap>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    if pos.x >= map.width || pos.y >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.pos);
        if position.pos == pos && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if tooltip.is_empty() {
        return;
    }

    let mut width: i32 = 0;
    for s in tooltip.iter() {
        if width < s.len() as i32 {
            width = s.len() as i32;
        }
    }
    width += 3;

    if pos.x > 40 {
        let arrow_pos = Point::new(pos.x - 2, pos.y);
        let left_x = pos.x - width;
        let mut y = pos.y;
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
        let arrow_pos = Point::new(pos.x + 1, pos.y);
        let left_x = pos.x + 3;
        let mut y = pos.y;
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
