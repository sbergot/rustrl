use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{
    components::*,
    constants::UI_HEIGHT,
    gamelog::GameLog,
    map::Map,
    player::{PlayerEntity, PlayerPos},
};

use super::gui_handlers::ItemUsage;

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
    let map = ecs.read_resource::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = pos;
    if mouse_pos.x >= map.width || mouse_pos.y >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.pos);
        if position.pos.x == mouse_pos.x && position.pos.y == mouse_pos.y && map.visible_tiles[idx]
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

        if mouse_pos.x > 40 {
            let arrow_pos = Point::new(mouse_pos.x - 2, mouse_pos.y);
            let left_x = mouse_pos.x - width;
            let mut y = mouse_pos.y;
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
            let arrow_pos = Point::new(mouse_pos.x + 1, mouse_pos.y);
            let left_x = mouse_pos.x + 3;
            let mut y = mouse_pos.y;
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
pub enum ItemMenuResult<T> {
    Cancel,
    NoResponse,
    Selected { result: T },
}

pub fn get_usage_options(ecs: &mut World, item: Entity) -> Vec<(String, ItemUsage)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut options = Vec::new();

    let consumable = ecs.read_storage::<Consumable>();
    if consumable.contains(item) {
        options.push(("use".to_string(), ItemUsage::Use));
    }

    let equippable = ecs.read_storage::<Equippable>();
    let equipped_storage = ecs.read_storage::<Equipped>();
    let equipped = equipped_storage.get(item);

    if let Some(equipped) = equipped {
        if equipped.owner == player_entity.entity {
            options.push(("unequip".to_string(), ItemUsage::Unequip));
        }
    } else {
        if equippable.contains(item) {
            options.push(("equip".to_string(), ItemUsage::Equip));
        }
    }

    options.push(("drop".to_string(), ItemUsage::Drop));

    options
}

pub fn get_inventory_options(ecs: &mut World) -> Vec<(String, Entity)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();
    let options: Vec<(String, Entity)> = (&backpack, &names, &entities)
        .join()
        .filter(|(item, _name, _e)| item.owner == player_entity.entity)
        .map(|(_i, name, entity)| (name.name.clone(), entity))
        .collect();
    options
}

pub fn get_equipped_options(ecs: &mut World) -> Vec<(String, Entity)> {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<Equipped>();
    let entities = ecs.entities();
    let options: Vec<(String, Entity)> = (&entities, &backpack, &names)
        .join()
        .filter(|(_entity, pack, _name)| pack.owner == player_entity.entity)
        .map(|(entity, _pack, name)| (name.name.clone(), entity))
        .collect();
    options
}

pub fn get_cells_in_range(ecs: &mut World, range: i32) -> Vec<Point> {
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
