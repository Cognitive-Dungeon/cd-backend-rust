use thiserror::Error;
use cd_core::ObjectGuid;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("entity {0:?} not found in registry")]
    EntityNotFound(ObjectGuid),

    #[error("entity {0:?} is missing component '{1}'")]
    MissingComponent(ObjectGuid, &'static str),

    #[error("movement blocked at {0:?}: tile is solid")]
    MovementBlocked(cd_core::WorldPos),
}