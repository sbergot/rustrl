use bracket_lib::prelude::{FontCharType, Point, RGB};
use serde::*;
#[allow(deprecated)]
use specs::{error::NoError, prelude::*, saveload::*, Entity};
use specs_derive::{Component, ConvertSaveload};

use crate::{
    entity_containers::{EntityHashMap, EntityVec},
    game_map::GameMap,
};

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub pos: Point,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

impl Viewshed {
    pub fn new(range: i32) -> Self {
        Self {
            visible_tiles: Vec::new(),
            dirty: true,
            range,
        }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub was_hurt: bool,
}

impl CombatStats {
    pub fn new(hp: i32, defense: i32, power: i32) -> CombatStats {
        CombatStats {
            max_hp: hp,
            hp,
            defense,
            power,
            was_hurt: false,
        }
    }

    pub fn deal_damage(&mut self, amount: i32) {
        self.hp -= amount;
        self.was_hurt = true;
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = i32::min(self.max_hp, self.hp + amount);
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Confused {
    pub turns: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Inventory {
    pub items: EntityVec<Entity>,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Hash, Eq)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipment {
    pub slots: EntityHashMap<EquipmentSlot, Entity>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, Serialize, Deserialize, PartialEq, Clone)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: GameMap,
}
