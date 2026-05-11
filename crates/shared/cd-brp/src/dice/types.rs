use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

impl DiceType {
    pub const fn faces(self) -> u32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
            DiceType::D100 => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sign {
    Negative,
    None,
    Positive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageModifier {
    pub sign: Sign,
    pub count: u32,
    pub dice: DiceType,
}

#[derive(Debug)]
pub struct ParseDamageModifierError;

impl FromStr for DamageModifier {
    type Err = ParseDamageModifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(ParseDamageModifierError)
    }
}

impl DamageModifier {
    pub const NONE: Self = Self {
        sign: Sign::None,
        count: 0,
        dice: DiceType::D6,
    };

    pub const fn new(sign: Sign, count: u32, dice: DiceType) -> Self {
        Self { sign, count, dice }
    }

    pub const fn is_none(&self) -> bool {
        self.count == 0 || matches!(self.sign, Sign::None)
    }

    /// Человеко-читаемое представление (для логов, UI, отладки)
    pub fn as_str(self) -> &'static str {
        use {DiceType::*, Sign::*};
        match (self.sign, self.count, self.dice) {
            (None, 0, _) => "0",
            (Negative, 1, D6) => "-1D6",
            (Negative, 1, D4) => "-1D4",
            (Positive, 1, D4) => "+1D4",
            (Positive, 1, D6) => "+1D6",
            (Positive, 2, D6) => "+2D6",
            (Positive, 3, D6) => "+3D6",
            (Positive, 4, D6) => "+4D6",
            // Fallback для кастомных значений
            (Negative, c, d) if c > 0 => {
                // В статической строке не можем форматировать, возвращаем заглушку
                "-XDY"
            }
            (Positive, c, d) if c > 0 => "+XDY",
            _ => "0",
        }
    }

    /// Парсинг из строки (для обратной совместимости / импорта)
    pub fn parse(s: &str) -> Option<Self> {
        use {DiceType::*, Sign::*};
        match s.trim().to_uppercase().as_str() {
            "0" | "" | "NONE" => Some(Self::NONE),
            "-1D6" => Some(Self::new(Negative, 1, D6)),
            "-1D4" => Some(Self::new(Negative, 1, D4)),
            "+1D4" => Some(Self::new(Positive, 1, D4)),
            "+1D6" => Some(Self::new(Positive, 1, D6)),
            "+2D6" => Some(Self::new(Positive, 2, D6)),
            "+3D6" => Some(Self::new(Positive, 3, D6)),
            "+4D6" => Some(Self::new(Positive, 4, D6)),
            _ => Some(Self::NONE),
        }
    }
}

impl Default for DamageModifier {
    fn default() -> Self {
        Self::NONE
    }
}
