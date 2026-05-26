use super::error::TypeError;
use crate::constants::{D100_MAX, D100_MIN};
use rand::Rng;
use rand::RngExt;
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
    pub(crate) const fn new(value: u16) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }

    pub fn roll<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self(rng.random_range(D100_MIN..=D100_MAX))
    }
}

impl fmt::Display for D100Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for D100Roll {
    type Err = crate::types::error::TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s
            .trim()
            .parse::<u16>()
            .map_err(|_| crate::types::error::TypeError::NegativeValue)?;
        Self::try_new(val)
    }
}

/// Строгий тип для броска улучшения навыка (Growth Roll).
// По умолчанию в BRP это 1D6, но мы допускаем расширение до 1D10 для талантов.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GrowthRoll(u8);

impl GrowthRoll {
    pub const MAX_GROWTH: u8 = 6; // Стандарт 6, но если берем запас под таланты то 10

    pub fn try_new(value: u8) -> Result<Self, TypeError> {
        if (1..=Self::MAX_GROWTH).contains(&value) {
            Ok(Self(value))
        } else {
            Err(TypeError::InvalidGrowthRoll {
                value,
                min: 1,
                max: Self::MAX_GROWTH,
            })
        }
    }

    #[inline]
    pub(crate) const fn new(value: u8) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(self) -> u16 {
        self.0 as u16
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

    pub fn roll<R: Rng + ?Sized>(self, rng: &mut R) -> u16 {
        rng.random_range(1..=self.faces())
    }
}

impl fmt::Display for DieType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug вывод для DieType::D6 это "D6", что нам идеально подходит
        write!(f, "{:?}", self)
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

impl DamageModifier {
    /// Генерирует итоговое значение модификатора урона.
    /// ВАЖНО: Возвращает `i16`, так как модификатор может быть отрицательным (например, -1D4)!
    pub fn roll<R: Rng + ?Sized>(&self, rng: &mut R) -> i16 {
        match self {
            Self::None => 0,
            Self::Modifier { sign, count, dice } => {
                let mut total: i16 = 0;
                for _ in 0..*count {
                    total += dice.roll(rng) as i16;
                }

                match sign {
                    ModifierSign::Positive => total,
                    ModifierSign::Negative => -total,
                }
            }
        }
    }
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

    /// Бросает кубики и прибавляет статический модификатор.
    /// Возвращает `u16`, так как базовый урон самого оружия (без учета Damage Modifier)
    /// не уходит в минус (он ограничивается 0).
    pub fn roll<R: Rng + ?Sized>(&self, rng: &mut R) -> u16 {
        let mut total: i32 = 0;
        for _ in 0..self.count {
            total += self.die.roll(rng) as i32;
        }

        total += self.flat_modifier as i32;

        // Урон от самого кубика оружия не может быть отрицательным
        total.max(0) as u16
    }
}

impl fmt::Display for DiceExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.flat_modifier == 0 {
            write!(f, "{}{}", self.count, self.die)
        } else if self.flat_modifier > 0 {
            write!(f, "{}{}+{}", self.count, self.die, self.flat_modifier)
        } else {
            // У flat_modifier уже есть знак минуса, так как это i16
            write!(f, "{}{}{}", self.count, self.die, self.flat_modifier)
        }
    }
}
