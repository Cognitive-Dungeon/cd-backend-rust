use serde::{Deserialize, Serialize};

use crate::HitPoints;

// --- ПОГОДА И СРЕДА (стр. 119-120) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindForce {
    Light,
    Strong,
    Severe,
    Windstorm,
    Hurricane,
    Tornado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Precipitation {
    None,
    Rain,
    Snow,
    Sleet, // Слякоть/мокрый снег
    Hail,  // Град
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudCover {
    Light,
    Heavy,
    Severe,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MedicalCareConditions {
    Poor,
    Decent,
    Excellent,
}

// --- БОЛЕЗНИ И РАДИАЦИЯ (стр. 96, 111) ---

/// Симптомы и эффекты болезней (привязаны к падению конкретной характеристики)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiseaseType {
    Atrophy,  // Бьет по STR
    Chills,   // Бьет по CON
    Delirium, // Бьет по INT
    Malaise,  // Бьет по POW
    Shakes,   // Бьет по DEX
    Pox,      // Бьет по CHA
}

pub enum DiseaseSeverity {
    None,
    Mild,
    Acute,
    Severe,
    Terminal,
}

/// Уровни радиационного облучения
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadiationIntensity {
    Low,
    Moderate,
    Medium,
    High,
    Acute,
    Fatal,
}

/// Тип удушья/асфиксии (стр. 92)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsphyxiationSource {
    Water,         // Утопление
    Smoke,         // Дым
    DenseSmoke,    // Густой дым
    PoisonGas,     // Ядовитый газ
    Strangulation, // Удушение веревкой/руками
    Vacuum,        // Вакуум (космос)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeedOfEffect {
    Instantaneous,
    CombatRounds(u16),
    Minutes(u16),
    Hours(u16),
    Days(u16),
}

/// Результат падения с высоты.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallingResult {
    /// Количество урона, которое получает персонаж.
    /// Стр. 94: Броня обычно НЕ защищает от урона от падения, кроме особых случаев.
    pub damage_taken: HitPoints,
    /// Произошло ли успешное смягчение падения (уменьшает урон).
    pub mitigated: bool,
}

/// Статус удушья в текущем раунде.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsphyxiationResult {
    /// Персонаж успешно задержал дыхание или перенес газ (урона нет).
    Safe,
    /// Персонаж не выдержал. Получает прямой урон в HP (броня не спасает).
    TakesDamage(HitPoints),
    /// Персонаж критически провалил проверку удушья. Падает в обморок.
    Unconscious,
}
