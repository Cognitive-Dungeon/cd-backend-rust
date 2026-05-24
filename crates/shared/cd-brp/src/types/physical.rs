// crates/shared/cd-brp/src/types/physical.rs
use serde::{Deserialize, Serialize};

/// Единицы измерения нагрузки (Encumbrance, стр. 31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct EncumbrancePoints(pub u16);

impl EncumbrancePoints {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }
}
