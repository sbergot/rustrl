use bracket_lib::prelude::{FontCharType, Point, RGB};
use serde::*;
#[allow(deprecated)]
use specs::{
    error::NoError,
    prelude::*,
    saveload::*,
    Entity,
};
use specs_derive::{Component, ConvertSaveload};

use crate::map::Map;

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
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {
                amount: vec![amount],
            };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToMove {
    pub target: Point,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item: Entity,
}

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

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : Map
}