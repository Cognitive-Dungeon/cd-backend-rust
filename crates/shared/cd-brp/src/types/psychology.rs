use serde::{Deserialize, Serialize};

use crate::{SanityPoints, types::DefId};

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

/// Цена встречи с ужасающим (Стр. 104).
/// В BRP обычно записывается как "0/1D6" или "1/1D10".
/// Сервер должен сам бросить кубики и передать сюда готовые значения.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanityCost {
    /// Потеря SAN при успешном броске (обычно 0 или 1).
    pub on_success: SanityPoints,
    /// Потеря SAN при проваленном броске (результат кубика, например, выпало 4 на 1D6).
    pub on_failure: SanityPoints,
}

impl SanityCost {
    pub const fn new(on_success: SanityPoints, on_failure: SanityPoints) -> Self {
        Self {
            on_success,
            on_failure,
        }
    }
}

/// Результат проверки рассудка
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SanityResolution {
    /// Фактически потерянное количество очков SAN
    pub points_lost: SanityPoints,
    /// Был ли бросок рассудка успешным (иногда важно для других эффектов)
    pub is_success: bool,
    /// Если true, персонаж потерял >= 5 SAN за раз.
    /// Требуется немедленный бросок Интеллекта (Idea Roll), чтобы избежать Временного Безумия!
    pub triggers_temporary_insanity_risk: bool,
    /// Если true, персонаж потерял >= 20% SAN за короткое время (час). Безумие наступает автоматически.
    pub triggers_indefinite_insanity: bool,
}
