use crate::components::*;
use bracket_lib::prelude::*;
use specs::*;

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 3;

pub fn player(ecs: &mut World, pos: Point) -> Entity {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed::new(8))
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

pub fn random_monster(ecs: &mut World, pos: Point) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(ecs, pos),
        _ => goblin(ecs, pos),
    }
}

fn orc(ecs: &mut World, pos: Point) {
    monster(ecs, pos, to_cp437('o'), "Orc");
}
fn goblin(ecs: &mut World, pos: Point) {
    monster(ecs, pos, to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(ecs: &mut World, pos: Point, glyph: FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph,
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<Point> = Vec::new();
    let mut item_spawn_points: Vec<Point> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let pos = Point { x, y };
                if !monster_spawn_points.contains(&pos) {
                    monster_spawn_points.push(pos);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let pos = Point { x, y };
                if !item_spawn_points.contains(&pos) {
                    item_spawn_points.push(pos);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for pos in monster_spawn_points.iter() {
        random_monster(ecs, *pos);
    }

    for pos in item_spawn_points.iter() {
        random_item(ecs, *pos);
    }
}

fn health_potion(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { heal_amount: 8 })
        .build();
}

fn magic_missile_scroll(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .build();
}

fn random_item(ecs: &mut World, pos: Point) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => health_potion(ecs, pos),
        _ => magic_missile_scroll(ecs, pos),
    }
}
