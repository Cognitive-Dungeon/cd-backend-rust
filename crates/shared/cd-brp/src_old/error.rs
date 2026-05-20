use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BrpError {
    #[error("Invalid dice modifier string: {0}")]
    InvalidModifier(String),

    #[error("Characteristic value out of range: {value} (expected 1-100)")]
    CharacteristicOutOfRange { value: i32 },

    #[error("Dice roll error: {0}")]
    RollError(String),
}

pub type BrpResult<T> = Result<T, BrpError>;
