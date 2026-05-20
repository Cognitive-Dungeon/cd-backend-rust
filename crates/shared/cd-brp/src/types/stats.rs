use super::markers::CharacteristicMarker;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

/// Обобщенный строгий тип для значения конкретной характеристики.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Stat<T: CharacteristicMarker> {
    value: u16,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T: CharacteristicMarker> Stat<T> {
    #[inline]
    pub const fn new(value: u16) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.value
    }
}

impl<T: CharacteristicMarker> fmt::Display for Stat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.value, T::ABBREVIATION)
    }
}

/// Строгий тип для рейтинга навыка (от 0 до бесконечности).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SkillRating(u16);

impl SkillRating {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }
}
