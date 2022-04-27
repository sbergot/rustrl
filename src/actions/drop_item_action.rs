use specs::{Entity, WorldExt};

use crate::{components::*, gamelog::GameLog};

use super::{has_component, Action};

pub struct DropItemAction {
    pub target: Entity,
}

impl Action for DropItemAction {
    fn run(&self, actor: Entity, ecs: &mut specs::World) {
        let mut storage = ecs.write_storage::<Inventory>();
        let inventory = storage.get_mut(actor).unwrap();

        if let Some(index) = inventory.items.iter().position(|ent| *ent == self.target) {
            inventory.items.remove(index);
        }

        if has_component::<Player>(ecs, actor) {
            let mut log = ecs.write_resource::<GameLog>();
            let storage = ecs.read_component::<Name>();
            let item_name = storage.get(self.target).unwrap().name.clone();
            log.log(format!("You drop up the {}.", item_name))
        }

        let mut storage = ecs.write_component::<Position>();
        let position = storage.get(actor).unwrap().clone();
        storage
            .insert(self.target, position)
            .expect("Unable to insert position");
    }
}
