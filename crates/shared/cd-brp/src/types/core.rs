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
