use bracket_lib::prelude::Point;
use specs::WorldExt;

use crate::{map::Map, components::*, player::PlayerPos};

use super::{Action, has_component};

pub struct MoveAction {
    pub target: Point,
}

impl Action for MoveAction {
    fn run(&self, actor: specs::Entity, ecs: &mut specs::World) {
        let mut map = ecs.write_resource::<Map>();
        let mut player_pos = ecs.write_resource::<PlayerPos>();

        let mut storage = ecs.write_storage::<Viewshed>();
        let viewshed = storage.get_mut(actor);
        let mut storage = ecs.write_storage::<Position>();
        let position = storage.get_mut(actor).unwrap();

        let is_player = has_component::<Player>(ecs, actor);

        let target_idx = map.xy_idx(self.target);
        if !map.blocked_tiles[target_idx] {
            let pos_idx = map.xy_idx(position.pos);
            map.blocked_tiles[pos_idx] = false;
            map.blocked_tiles[target_idx] = true;
            position.pos = self.target;

            if is_player {
                player_pos.pos = self.target;
            }

            if let Some(mut viewshed) = viewshed {
                viewshed.dirty = true;
            }
        }
    }
}