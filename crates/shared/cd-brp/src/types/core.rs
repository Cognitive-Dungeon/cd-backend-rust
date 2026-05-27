use serde::{Deserialize, Serialize};

use crate::types::dice::DiceExpression;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PowerLevel {
    #[default]
    Normal,
    Heroic,
    Epic,
    Superhuman,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessLevel {
    Fumble,          // Крит. провал (обычно 99-00)
    Failure,         // Провал (бросок > шанса)
    Success,         // Успех (бросок <= шанса)
    SpecialSuccess,  // Особый успех (бросок <= 1/5 шанса)
    CriticalSuccess, // Критический успех (бросок <= 1/20 шанса)
}

impl SuccessLevel {
    /// Проверка на любой положительный успех
    pub const fn is_success(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::SpecialSuccess | Self::CriticalSuccess
        )
    }

    /// Проверка на провал
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failure | Self::Fumble)
    }
}

/// Результат встречной проверки (Opposed Roll, стр. 26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpposedOutcome {
    ActiveWins(SuccessLevel),
    PassiveWins(SuccessLevel),
    Tie, // Ничья (редко, но бывает)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyModifier {
    Automatic,
    Easy,
    Average,
    Difficult,
    Extreme,
    Impossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Combat,
    Communication,
    Manipulation,
    Mental,
    Perception,
    Physical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeScale {
    CombatRound,
    Turn,
    Scene,
    Narrative,
}

/// Представление значения Брони (AV).
/// По правилам (стр. 174) броня может быть фиксированной (AV 7) или рандомной (Random AV 1D8-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ArmorValue {
    Fixed(u16),
    Random(DiceExpression),
}

/// Базовое расстояние, проходимое за раунд (Movement, стр. 30).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct MovementRate(pub u16);

/// Скорость техники и маунтов (Rated Speed, стр. 202)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct RatedSpeed(pub u16);

/// Способ передвижения существа (стр. 513)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    Walk,
    Swim,
    Fly,
    Slither,
    Burrow,
}

/// Строгий тип для расстояния в метрах.
/// В BRP (и метрической системе) базовая единица дистанции.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Meters(pub u32);

impl Meters {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(val: u32) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Строгий тип для количества Боевых Раундов (Combat Rounds).
/// В BRP 1 раунд = 12 секунд. 5 раундов = 1 минута.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct CombatRounds(pub u32);

impl CombatRounds {
    pub const ZERO: Self = Self(0);

    /// Конвертация игровых минут в боевые раунды (1 мин = 5 раундов).
    pub const fn from_minutes(minutes: u32) -> Self {
        Self(minutes.saturating_mul(5))
    }

    #[inline]
    pub const fn new(val: u32) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Строгий тип для базовой стоимости предмета (Base Value).
/// В MMO/VTT представляет универсальную минимальную единицу валюты сеттинга
/// (например, медные монеты, кредиты, центы).
/// Опционально для классического BRP, но критически важно для машинной реализации торговли.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct Currency(u32); // Используем u32, так как экономика MMO требует больших чисел

impl Currency {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(val: u32) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Безопасное умножение (например, для продажи группы предметов или наценок)
    #[inline]
    pub const fn saturating_mul(self, multiplier: u32) -> Self {
        Self(self.0.saturating_mul(multiplier))
    }

    /// Безопасное деление (например, торговцы скупают лут за 50% цены)
    #[inline]
    pub const fn saturating_div(self, divisor: u32) -> Self {
        if divisor == 0 {
            Self::ZERO
        } else {
            Self(self.0 / divisor)
        }
    }
}

impl std::ops::Add for Currency {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl std::ops::Sub for Currency {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl std::ops::AddAssign for Currency {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl std::ops::SubAssign for Currency {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

// Суммирование коллекций (для подсчета стоимости всего инвентаря)
impl std::iter::Sum for Currency {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, val| acc + val)
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
