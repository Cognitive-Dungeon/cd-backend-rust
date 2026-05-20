use crate::types::core::PowerLevel;
use serde::{Deserialize, Serialize};

/// Режим расчета максимальных Hit Points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HpCalculationRule {
    /// HP = ceil((CON + SIZ) / 2)
    Average,
    /// HP = CON + SIZ (Опциональное правило для высокой выживаемости)
    Total,
}

/// Режим учета усталости
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FatigueRule {
    /// Игнорировать усталость
    None,
    /// Детальный подсчет очков (FP)
    DetailedPoints,
    /// Упрощенная система (Fresh -> Fatigued -> Severe -> Exhausted)
    SimpleStates,
}

/// Глобальная конфигурация правил для текущей игры
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSessionConfig {
    pub power_level: PowerLevel,
    pub hp_calculation: HpCalculationRule,
    pub fatigue_rule: FatigueRule,

    pub use_hit_locations: bool,
    pub use_education_stat: bool,
    pub use_skill_category_bonuses: bool,
    pub use_sanity: bool,

    /// True: INT * 10 (Normal), False: зависят от PowerLevel (INT * 15/20/25)
    pub use_increased_personal_skills: bool,
}
