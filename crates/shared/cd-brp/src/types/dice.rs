use super::error::TypeError;
use crate::constants::{D100_MAX, D100_MIN};
use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Строгий тип для броска D100. Гарантирует, что значение всегда находится в диапазоне 1..=100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl rand::distr::Distribution<D100Roll> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> D100Roll {
        D100Roll::new(rng.random_range(D100_MIN..=D100_MAX))
    }
}

// Сериализация/Десериализация D100Roll как простого числа с валидацией
impl Serialize for D100Roll {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

impl<'de> Deserialize<'de> for D100Roll {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u16::deserialize(deserializer)?;
        D100Roll::try_new(value).map_err(serde::de::Error::custom)
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

impl rand::distr::Distribution<u16> for DieType {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u16 {
        rng.random_range(1..=self.faces())
    }
}

impl std::str::FromStr for DieType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "D2" => Ok(Self::D2),
            "D3" => Ok(Self::D3),
            "D4" => Ok(Self::D4),
            "D6" => Ok(Self::D6),
            "D8" => Ok(Self::D8),
            "D10" => Ok(Self::D10),
            "D12" => Ok(Self::D12),
            "D20" => Ok(Self::D20),
            "D100" => Ok(Self::D100),
            _ => Err(format!("Unknown die type: {}", s)),
        }
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

impl rand::distr::Distribution<i16> for DamageModifier {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> i16 {
        match self {
            Self::None => 0,
            Self::Modifier { sign, count, dice } => {
                let mut total: i16 = 0;
                for _ in 0..*count {
                    total += rng.random_range(1..=dice.faces()) as i16;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Максимально возможный результат (для расчета Critical Damage).
    pub const fn max_value(&self) -> u16 {
        let max_dice = (self.count as u32).saturating_mul(self.die.faces() as u32);
        let total = (max_dice as i32).saturating_add(self.flat_modifier as i32);
        if total < 0 { 0 } else { total as u16 }
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

impl rand::distr::Distribution<u16> for DiceExpression {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u16 {
        let mut total: i32 = 0;
        for _ in 0..self.count {
            total += rng.random_range(1..=self.die.faces()) as i32;
        }
        total += self.flat_modifier as i32;
        total.max(0) as u16 // Урон оружия не может быть отрицательным
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

impl std::str::FromStr for DiceExpression {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace(" ", "").to_uppercase(); // Убираем пробелы и приводим к "1D8+2"

        let _split_char = if s.contains('+') {
            '+'
        } else if s.contains('-') {
            '-'
        } else {
            'D'
        };

        // Разбираем модификатор
        let (dice_part, modifier) = if s.contains('+') {
            let parts: Vec<&str> = s.split('+').collect();
            (
                parts[0],
                parts.get(1).unwrap_or(&"0").parse::<i16>().unwrap_or(0),
            )
        } else if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            (
                parts[0],
                -parts.get(1).unwrap_or(&"0").parse::<i16>().unwrap_or(0),
            )
        } else {
            (s.as_str(), 0)
        };

        // Разбираем кубик (например "2D6" или просто "D100")
        let d_parts: Vec<&str> = dice_part.split('D').collect();
        if d_parts.len() != 2 {
            return Err(format!("Invalid dice format: {}", s));
        }

        let count = if d_parts[0].is_empty() {
            1
        } else {
            d_parts[0].parse::<u8>().unwrap_or(1)
        };
        let die_str = format!("D{}", d_parts[1]);
        let die = DieType::from_str(&die_str)?;

        Ok(Self::new(count, die, modifier))
    }
}

impl Serialize for DiceExpression {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DiceExpression {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        <DiceExpression as std::str::FromStr>::from_str(&s).map_err(serde::de::Error::custom)
    }
}
