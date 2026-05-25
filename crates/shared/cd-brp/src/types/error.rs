use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    #[error("D100 roll must be between {min} and {max}, got {value}")]
    InvalidD100Roll { value: u16, min: u16, max: u16 },

    #[error("Growth roll must be between {min} and {max}, got {value}")]
    InvalidGrowthRoll { value: u8, min: u8, max: u8 },

    #[error("Value cannot be negative")]
    NegativeValue,
}
