mod dispatcher;
pub use dispatcher::UnifiedDispatcher;

// System imports
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod ai;
use ai::*;
mod movement_system;
use movement_system::MovementSystem;
mod trigger_system;
use trigger_system::TriggerSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod ranged_combat_system;
use ranged_combat_system::RangedCombatSystem;
mod inventory_system;
use inventory_system::*;
mod hunger_system;
use hunger_system::HungerSystem;
pub mod particle_system;
use particle_system::ParticleSpawnSystem;
mod lighting_system;
use lighting_system::LightingSystem;

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    dispatcher::new()
}