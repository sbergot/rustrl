use specs::{Entity, WorldExt};

use crate::{components::*, gamelog::GameLog};

use super::{has_component, Action};

pub struct EquipItemAction {
    pub target: Entity,
}

impl Action for EquipItemAction {
    fn run(&self, actor: Entity, ecs: &mut specs::World) {
        let is_player = has_component::<Player>(ecs, actor);
        let mut log = ecs.write_resource::<GameLog>();

        let mut storage = ecs.write_storage::<Inventory>();
        let inventory = storage.get_mut(actor).unwrap();

        let storage = ecs.write_component::<Equippable>();
        let can_equip = storage.get(self.target).unwrap();
        let target_slot = can_equip.slot;

        // Remove any items the target has in the item's slot
        let mut storage = ecs.write_component::<Equipment>();
        let equipment = storage.get_mut(actor).unwrap();
        let to_unequip = equipment.slots.get(&target_slot).cloned();

        if let Some(to_unequip) = to_unequip {
            if is_player {
                let storage = ecs.read_component::<Name>();
                let name = storage.get(to_unequip).unwrap();
                log.log(format!("You unequip {}.", name.name));
            }
            equipment.slots.remove(&target_slot);
            inventory.items.push(to_unequip);
        }

        equipment.slots.insert(target_slot, self.target);
        if let Some(index) = inventory.items.iter().position(|ent| *ent == self.target) {
            inventory.items.remove(index);
        }

        if is_player {
            let storage = ecs.read_component::<Name>();
            let item_name = storage.get(self.target).unwrap().name.clone();
            log.log(format!("You eauip the {}.", item_name))
        }
    }
}
