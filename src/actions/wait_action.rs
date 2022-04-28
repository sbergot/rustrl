use specs::Entity;

use super::Action;

pub struct WaitAction {}

impl Action for WaitAction {
    fn run(&self, _actor: Entity, _ecs: &mut specs::World) {}
}
