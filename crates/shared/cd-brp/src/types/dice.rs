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

impl DieType {
    pub const fn faces(self) -> u16 {
        match self {
            Self::D2 => 2,
            Self::D3 => 3,
            Self::D4 => 4,
            Self::D6 => 6,
            Self::D8 => 8,
            Self::D10 => 10,
            Self::D12 => 12,
            Self::D20 => 20,
            Self::D100 => 100,
        }
    }
}

/// Знак модификатора (положительный или отрицательный)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModifierSign {
    Negative,
    Positive,
}

/// Модификатор урона персонажа (Damage Modifier, стр. 34-35 рулбука).
/// Пример: None, Modifier { sign: Negative, count: 1, dice: D4 } -> "-1D4"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum DamageModifier {
    #[default]
    None,
    Modifier {
        sign: ModifierSign,
        count: u8,
        dice: DieType,
    },
}

impl fmt::Display for DamageModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "0"),
            Self::Modifier { sign, count, dice } => {
                let s = match sign {
                    ModifierSign::Negative => "-",
                    ModifierSign::Positive => "+",
                };
                write!(f, "{}{}{:?}", s, count, dice)
            }
        }
    }
}

/// Строгая структура для вычисления выражений вида "XDY + Z" (напр. 2D6 + 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceExpression {
    pub count: u8,
    pub die: DieType,
    pub flat_modifier: i16,
}

impl DiceExpression {
    pub const fn new(count: u8, die: DieType, flat_modifier: i16) -> Self {
        Self {
            count,
            die,
            flat_modifier,
        }
    }
}
