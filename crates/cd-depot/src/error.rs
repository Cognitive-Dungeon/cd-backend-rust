use thiserror::Error;

#[derive(Debug, Error)]
pub enum DepotError {
    #[error("io error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("sheet '{0}' not found in depot file")]
    SheetNotFound(String),
}