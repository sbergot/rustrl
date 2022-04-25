mod move_action;

use specs::{World, Entity};

pub trait Action {
    fn run(&self, actor: Entity, ecs: &mut World);
}