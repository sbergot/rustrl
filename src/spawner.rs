use std::collections::HashMap;

use crate::{
    components::*,
    entity_containers::{EntityHashMap, EntityVec}, random_table::RandomTable,
};
use bracket_lib::prelude::*;
use specs::{saveload::*, *};

const MAX_SPAWNED: i32 = 5;

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
        .with(CombatStats::new(30, 2, 5))
        .with(Inventory {
            items: EntityVec::new(),
        })
        .with(Equipment {
            slots: EntityHashMap::new(),
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
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
        .with(CombatStats::new(16, 1, 4))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(mut ecs: &mut World, room: &Rect) {
    let mut rng = {
        ecs.write_resource::<RandomNumberGenerator>().clone()
    };
    

    let mut spawn_table = room_table(&mut rng);
    let mut spawn_points: HashMap<Point, ()> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_SPAWNED + 3) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let pos = Point { x, y };
                if !spawn_points.contains_key(&pos) {
                    spawn_points.insert(pos, ());
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    {
        for (spawn_pos, _u) in spawn_points.iter() {
            let spawner = spawn_table.roll();
            spawner(&mut ecs, *spawn_pos);
        }
    }
}

pub type Spawner = fn(ecs: &mut World, pos: Point);

fn room_table(rng: &mut RandomNumberGenerator) -> RandomTable<Spawner> {
    RandomTable::<Spawner>::new(rng)
        .add(goblin, 20)
        .add(orc, 5)
        .add(health_potion, 7)
        .add(fireball_scroll, 2)
        .add(confusion_scroll, 2)
        .add(magic_missile_scroll, 4)
        .add(dagger, 3)
        .add(shield, 3)
        .add(longsword, 2)
        .add(tower_shield, 2)
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
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(ORANGE),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(PINK),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437('/'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437('('),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437('/'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Longsword".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, pos: Point) {
    ecs.create_entity()
        .with(Position { pos })
        .with(Renderable {
            glyph: to_cp437('('),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Tower Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
