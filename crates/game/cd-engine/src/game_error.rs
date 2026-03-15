use thiserror::Error;
use cd_core::{ObjectGuid, WorldPos};

#[derive(Debug, Error)]
pub enum GameError {
    #[error("entity {0:?} not found")]
    EntityNotFound(ObjectGuid),

    #[error("entity {guid:?} missing component '{component}'")]
    MissingComponent { guid: ObjectGuid, component: &'static str },

    #[error("movement blocked at {0:?}: tile is solid")]
    MovementBlocked(WorldPos),

    #[error("operation not permitted: {0}")]
    NotPermitted(String),
}

/// Результат нанесения урона
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub actual_damage: i32,
    pub killed: bool,
}