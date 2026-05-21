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
    Fumble,
    Failure,
    Success,
    SpecialSuccess,
    CriticalSuccess,
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
