mod melee_action;
pub use melee_action::*;
mod move_action;
pub use move_action::*;
mod pick_item_action;
pub use pick_item_action::*;
mod drop_item_action;
pub use drop_item_action::*;
mod equip_item_action;
pub use equip_item_action::*;
mod unequip_item_action;
pub use unequip_item_action::*;
mod use_item_action;
pub use use_item_action::*;

use specs::*;

use crate::components::Equipment;

pub trait Action {
    fn run(&self, actor: Entity, ecs: &mut World);
}

pub type AnyAction = Option<Box<dyn Action>>;

pub fn has_component<T: Component>(ecs: &World, entity: Entity) -> bool {
    ecs.read_storage::<T>().contains(entity)
}

pub fn map_equipped_items_comp<T: Component, R>(
    ecs: &World,
    owner: Entity,
    map: fn(&T) -> R,
) -> Vec<R> {
    let equippement_storage = ecs.read_storage::<Equipment>();
    let equipment = equippement_storage.get(owner);
    if let Some(equipment) = equipment {
        let equipment = equipment.slots.values();
        let storage = ecs.read_storage::<T>();
        equipment
            .filter_map(|entity| {
                let component = storage.get(*entity)?;
                Some(map(component))
            })
            .collect()
    } else {
        Vec::new()
    }
}
