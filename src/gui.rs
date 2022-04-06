use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{components::*, constants::UI_HEIGHT, gamelog::GameLog, map::Map};

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
}
