use serde::{Deserialize, Serialize};

use crate::types::DefId;

/// Приверженность (Стр. 493). Отражает служение Свету, Хаосу, богам.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AllegianceId(pub DefId);

/// Длительность Временного Безумия (стр. 509)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsanityDuration {
    FiveMinutes,
    OneHour,
    TwoHours,
    TwelveHours,
    OneDay,
    TwoDays,
    OneWeek,
}

/// Длительность Отчаяния (Despair) при провале Страсти (стр. 502)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DespairDuration {
    CombatRounds1D10,
    CombatRoundsD10Plus10,
    UntilSunriseOrSunset,
    GameDays1D3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryInsanityType {
    Catatonia,
    Stupefaction,
    Paranoia,
    Phobia,
    Amnesia,
    SuicidalDespondency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassionType {
    Devotion,
    Fear,
    Hate,
    Honour,
    Love,
    Loyalty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalityTraitPair {
    AggressivePassive,
    ImpulsiveCautious,
    ExtrovertIntrovert,
    OptimisticPessimistic,
    StubbornReceptive,
    PhysicalMental,
    PatientNervous,
    EmotionalCalm,
    TrustingSuspicious,
    LeaderFollower,
    GreedyGenerous,
    EnergeticLazy,
    HonourableDishonourable,
    BraveCowardly,
    CuriousIncurious,
    DependableUnreliable,
    PiousIrreligious,
    HonestDishonest,
    CleverDull,
    HumorousDour,
    ConservativeInnovative,
}
