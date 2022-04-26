use specs::{Entity, WorldExt};

use crate::{components::*, gamelog::GameLog};

use super::{has_component, Action};

pub struct UnequipItemAction {
    pub target: Entity,
}

impl Action for UnequipItemAction {
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
        let to_unequip = equipment.slots.get(&target_slot);
        assert!(to_unequip == Some(&self.target));

        if is_player {
            let storage = ecs.read_component::<Name>();
            let name = storage.get(self.target).unwrap();
            log.log(format!("You unequip {}.", name.name));
        }
        equipment.slots.remove(&target_slot);
        inventory.items.push(self.target);
    }
}
