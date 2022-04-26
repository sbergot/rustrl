mod visibility_system;
pub use visibility_system::*;

mod monster_ai_system;
pub use monster_ai_system::*;

mod map_indexing_system;
pub use map_indexing_system::*;

mod dead_collection_system;
pub use dead_collection_system::*;

mod saveload_system;
pub use saveload_system::*;

mod points_of_interest_system;
pub use points_of_interest_system::*;

pub mod particle_system;
pub use particle_system::*;

use specs::DispatcherBuilder;

pub fn with_gameplay_systems<'a, 'b>(
    dispatcher: DispatcherBuilder<'a, 'b>,
) -> DispatcherBuilder<'a, 'b> {
    dispatcher
        .with(ParticleSpawnSystem {}, "particle", &[])
        .with(VisibilitySystem {}, "visibility", &[])
        .with(DeadCollection {}, "dead_collection", &[])
}

pub fn with_indexing_systems<'a, 'b>(
    dispatcher: DispatcherBuilder<'a, 'b>,
) -> DispatcherBuilder<'a, 'b> {
    dispatcher
        .with(MapIndexingSystem {}, "map_indexing", &[])
        .with(
            PointsOfInterestSystem {},
            "points_of_interest",
            &["map_indexing"],
        )
}
