use bracket_lib::prelude::Point;
use specs::WorldExt;

use crate::{map::Map, components::*, player::PlayerPos};

use super::Action;

pub struct MoveAction {
    target: Point,
}

impl Action for MoveAction {
    fn run(&self, actor: specs::Entity, ecs: &mut specs::World) {
        let mut map = ecs.write_resource::<Map>();
        let mut player_pos = ecs.write_resource::<PlayerPos>();

        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let mut positions = ecs.write_storage::<Position>();
        let players = ecs.read_storage::<Player>();

        let mut position = positions.get_mut(actor).unwrap();
        
        let target_idx = map.xy_idx(self.target);
        if !map.blocked_tiles[target_idx] {
            let pos_idx = map.xy_idx(position.pos);
            map.blocked_tiles[pos_idx] = false;
            map.blocked_tiles[target_idx] = true;
            position.pos = self.target;

            if players.contains(actor) {
                player_pos.pos = self.target;
            }

            if let Some(mut viewshed) = viewsheds.get_mut(actor) {
                viewshed.dirty = true;
            }
        }
    }
}