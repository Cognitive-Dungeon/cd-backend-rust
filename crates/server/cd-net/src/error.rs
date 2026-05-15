use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("websocket error: {0}")]
    Ws(#[from] axum::Error),

    #[error("engine communication failed")]
    EngineDead,

    #[error("unauthorized: login required")]
    Unauthorized,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type NetResult<T> = Result<T, NetError>;
