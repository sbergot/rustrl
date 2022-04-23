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

mod dead_collection_system;
pub use dead_collection_system::*;

mod inventory_system;
pub use inventory_system::*;

mod movement_system;
pub use movement_system::*;

mod saveload_system;
pub use saveload_system::*;

mod points_of_interest_system;
pub use points_of_interest_system::*;

pub mod particle_system;
pub use particle_system::*;

use specs::DispatcherBuilder;

pub fn with_systems<'a, 'b>(dispatcher: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
    dispatcher
        .with(ParticleSpawnSystem {}, "particle", &[])
        .with(VisibilitySystem {}, "visibility", &[])
        .with(MonsterAI {}, "monster_ai", &["visibility"])
        .with(MovementSystem {}, "movement", &["monster_ai"])
        .with(MapIndexingSystem {}, "map_indexing", &["monster_ai"])
        .with(MeleeCombatSystem {}, "melee_combat", &["monster_ai"])
        .with(ItemCollectionSystem {}, "item_collection", &[])
        .with(ItemUseSystem {}, "potion_use", &[])
        .with(ItemDropSystem {}, "item_drop", &[])
        .with(ItemRemoveSystem {}, "item_remove", &[])
        .with(DamageSystem {}, "damage", &["melee_combat"])
        .with(VisibilitySystem {}, "visibility2", &["movement"])
        .with(PointsOfInterestSystem {}, "points_of_interest", &["movement"])
        .with(DeadCollection {}, "dead_collection", &["damage"])
}
