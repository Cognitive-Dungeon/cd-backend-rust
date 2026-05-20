use super::error::TypeError;
use crate::constants::{D100_MAX, D100_MIN};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Строгий тип для броска D100. Гарантирует, что значение всегда находится в диапазоне 1..=100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct D100Roll(u16);

impl D100Roll {
    pub fn try_new(value: u16) -> Result<Self, TypeError> {
        if (D100_MIN..=D100_MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(TypeError::InvalidD100Roll {
                value,
                min: D100_MIN,
                max: D100_MAX,
            })
        }
    }

    #[inline]
    pub(crate) const fn new_unchecked(value: u16) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for D100Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Типы классических кубиков
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DieType {
    D2,
    D3,
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

/// Строгая структура для вычисления выражений вида "XDY + Z" (напр. 2D6 + 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceExpression {
    pub count: u8,
    pub die: DieType,
    pub modifier: i16,
}

impl DiceExpression {
    pub const fn new(count: u8, die: DieType, modifier: i16) -> Self {
        Self {
            count,
            die,
            modifier,
        }
    }
}
