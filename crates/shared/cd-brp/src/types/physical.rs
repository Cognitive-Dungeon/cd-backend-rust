// crates/shared/cd-brp/src/types/physical.rs
use serde::{Deserialize, Serialize};

/// Состояния усталости персонажа (Fatigue стр. 33).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FatigueState {
    /// FP > 0. Персонаж полон сил, штрафов нет.
    #[default]
    Normal,
    /// FP <= 0. Персонаж утомлен. Все проверки навыков становятся Difficult (×½).
    Fatigued,
    /// FP <= -Max FP. Персонаж истощен. Он падает без сознания или не может действовать.
    Unconscious,
}
