use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use cd_core::WorldPos;

/// Намерение сущности переместиться.
#[derive(Debug, Clone, Message)]
pub struct IntentMove {
    pub entity: Entity,
    pub target: WorldPos,
}
