use specs::{Entity, WorldExt};

use crate::{components::*, gamelog::GameLog};

use super::{has_component, Action};

pub struct PickItemAction {
    pub target: Entity,
}

impl Action for PickItemAction {
    fn run(&self, actor: Entity, ecs: &mut specs::World) {
        let mut storage = ecs.write_storage::<Inventory>();
        let inventory = storage.get_mut(actor).unwrap();
        inventory.items.push(self.target);

        if has_component::<Player>(ecs, actor) {
            let mut log = ecs.write_resource::<GameLog>();
            let storage = ecs.read_component::<Name>();
            let item_name = storage.get(self.target).unwrap().name.clone();
            log.log(format!("You pick up the {}.", item_name))
        }

        let mut storage = ecs.write_component::<Position>();
        storage.remove(self.target);
    }
}
