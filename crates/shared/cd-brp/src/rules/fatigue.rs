//! Модуль системы Усталости (Fatigue, стр. 33).

use crate::rules::encumbrance::EncumbrancePenalties;
use crate::rules::modifiers::apply_difficulty;
use crate::types::{
    Con, DifficultyModifier, FatiguePoints, FatigueRule, FatigueState, SkillRating, Stat, Str,
};

/// Вычисляет максимальный запас очков усталости (Max FP).
/// Стр. 33: "Fatigue points (FP) are equal to your character’s STR+CON".
#[must_use]
pub const fn calculate_max_fp(str_stat: Stat<Str>, con_stat: Stat<Con>) -> FatiguePoints {
    let total = str_stat.get().saturating_add(con_stat.get());
    FatiguePoints::new(total as i16)
}

/// Вычисляет текущее состояние усталости на основе текущих FP и Max FP.
#[must_use]
pub const fn determine_fatigue_state(
    current_fp: FatiguePoints,
    max_fp: FatiguePoints,
    rule: FatigueRule,
) -> FatigueState {
    match rule {
        // Если правило отключено, персонаж всегда свеж
        FatigueRule::None => FatigueState::Normal,

        // В BRP UGE:
        // Если FP падает до 0 или ниже -> Fatigued.
        // Если FP достигает отрицательного Max FP (например, -25 при максимуме 25) -> Unconscious.
        FatigueRule::DetailedPoints | FatigueRule::SimpleStates => {
            let fp = current_fp.get();
            let max = max_fp.get();

            if fp > 0 {
                FatigueState::Normal
            } else if fp > -max {
                FatigueState::Fatigued
            } else {
                FatigueState::Unconscious
            }
        }
    }
}

/// Применяет штраф усталости к шансу навыка.
/// Стр. 33: "A fatigued character has all of their skill chances halved (Difficult)."
#[must_use]
pub fn apply_fatigue_penalty(base_skill: SkillRating, state: FatigueState) -> SkillRating {
    match state {
        FatigueState::Normal => base_skill,
        FatigueState::Fatigued => apply_difficulty(base_skill, DifficultyModifier::Difficult),
        // В состоянии Unconscious персонаж падает без сознания.
        // Проводить проверки навыков невозможно.
        FatigueState::Unconscious => SkillRating::ZERO,
    }
}

/// Рассчитывает, сколько FP персонаж теряет за 1 боевой раунд (или эквивалент нагрузки).
/// Учитывает перегруз (Encumbrance).
#[must_use]
pub const fn calculate_combat_round_fp_drain(
    enc_penalties: Option<EncumbrancePenalties>,
) -> FatiguePoints {
    // Базовая потеря в бою: 1 FP за раунд активных действий (атака, уворот и т.д.)
    let mut drain: i16 = 1;

    // Плюс штраф за перегруз (1 FP за каждый лишний ENC)
    if let Some(pen) = enc_penalties {
        drain = drain.saturating_add(pen.fatigue_drain_per_turn() as i16);
    }

    FatiguePoints::new(drain)
}
