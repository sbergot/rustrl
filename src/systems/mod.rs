mod visibility_system;
pub use visibility_system::*;

mod monster_ai_system;
pub use monster_ai_system::*;

mod map_indexing_system;
pub use map_indexing_system::*;

mod melee_combat_system;
pub use melee_combat_system::*;

mod damage_system;
pub use damage_system::*;

mod inventory_system;
pub use inventory_system::*;

mod movement_system;
pub use movement_system::*;

use specs::DispatcherBuilder;

pub fn with_systems<'a, 'b>(dispatcher: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
    dispatcher
        .with(VisibilitySystem {}, "visibility", &[])
        .with(MonsterAI {}, "monster_ai", &["visibility"])
        .with(MovementSystem {}, "movement", &["monster_ai"])
        .with(MapIndexingSystem {}, "map_indexing", &["monster_ai"])
        .with(MeleeCombatSystem {}, "melee_combat", &["monster_ai"])
        .with(ItemCollectionSystem {}, "item_collection", &[])
        .with(ItemUseSystem {}, "potion_use", &[])
        .with(ItemDropSystem {}, "item_drop", &[])
        .with(DamageSystem {}, "damage", &["melee_combat"])
        .with(VisibilitySystem {}, "visibility2", &["movement"])
}
