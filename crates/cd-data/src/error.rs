use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialize error: {0}")]
    Serialize(String),

    #[error("deserialize error: {0}")]
    Deserialize(String),

    #[error("key not found: {0}")]
    NotFound(String),
}