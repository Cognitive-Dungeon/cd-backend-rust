//! Модуль улучшения навыков (Skill Improvement / Experience, стр. 45-47).

use crate::{
    Edu, Int, Stat,
    dice::GrowthRoll,
    progression::{ExperienceBonus, MasteryTarget},
    types::{D100Roll, SkillRating},
};

/// Результат попытки улучшения навыка.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceRollResult {
    /// Бросок D100 оказался меньше или равен текущему навыку. Навык не вырос.
    NoImprovement,
    /// Бросок D100 превысил текущий навык (или это INT roll для навыков 100%+).
    /// Навык вырос на указанное количество пунктов (обычно 1D6).
    ImprovedBy(u16),
}

/// Выполняет проверку на улучшение навыка (Experience Roll, стр. 45).
///
/// `current_rating` — текущий шанс навыка персонажа.
/// `experience_roll` — бросок D100.
/// `growth_roll` — бросок кубика роста (обычно 1D6, генерируется сервером заранее).
/// `experience_bonus` — бонус опыта от Интеллекта (INT / 2, стр. 34).
///
/// # Правила BRP:
/// - Если навык < 100%, цель (Target) = текущее значение навыка.
/// - Если навык >= 100%, цель (Target) = `mastery_target` (обычно INT×5 или EDU×5).
/// - Эффективный бросок = `experience_roll` + `experience_bonus` (если игрок умный).
/// - Если эффективный бросок СТРОГО БОЛЬШЕ цели, навык растет на `growth_roll` (1D6).
#[must_use]
pub fn resolve_experience_roll(
    current_rating: SkillRating,
    experience_roll: D100Roll,
    growth_roll: GrowthRoll,           // Результат броска 1D6
    experience_bonus: ExperienceBonus, // Из DerivedStats.experience_bonus
    mastery_target: MasteryTarget,     // INT×5 или EDU×5
) -> ExperienceRollResult {
    let skill_value = current_rating.get();
    let roll_value = experience_roll.get();

    // 1. Определение порога (Target) для проверки
    let target = if skill_value >= 100 {
        mastery_target.get()
    } else {
        skill_value
    };

    // 2. Проверка на улучшение.
    // По правилам (стр. 45): игрок должен выкинуть БОЛЬШЕ своего текущего навыка.
    // Умные персонажи добавляют свой Experience Bonus к результату D100!
    let effective_roll = roll_value.saturating_add(experience_bonus.get());

    if effective_roll > target {
        // Успех! Навык растет на бросок 1D6 (growth_roll).
        ExperienceRollResult::ImprovedBy(growth_roll.get())
    } else {
        // Провал. Навык остается прежним.
        ExperienceRollResult::NoImprovement
    }
}

/// Применяет результат роста к навыку, возвращая новый рейтинг.
/// В BRP (стр. 45) навыки могут расти свыше 100%,
/// поэтому мы не ограничиваем рост искусственно (D100_MAX тут не применяется).
#[must_use]
pub fn apply_improvement(current_rating: SkillRating, result: ExperienceRollResult) -> SkillRating {
    match result {
        ExperienceRollResult::NoImprovement => current_rating,
        ExperienceRollResult::ImprovedBy(growth) => {
            // Теоретический кап для u16, но в реальности навыки редко уходят за 200-300%
            SkillRating::new(current_rating.get().saturating_add(growth))
        }
    }
}

/// Вычисляет порог проверки (Mastery Target) для навыков 100%+.
/// Стр. 46: Проверка идет против INTx5 (или EDUx5, если так решит GM).
#[must_use]
pub const fn calculate_mastery_target(
    int: Stat<Int>,
    edu: Option<Stat<Edu>>,
    use_edu: bool,
) -> MasteryTarget {
    let base_stat = if use_edu {
        if let Some(e) = edu {
            e.get()
        } else {
            int.get()
        }
    } else {
        int.get()
    };

    MasteryTarget::new(base_stat.saturating_mul(5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_improvement_success() {
        // Навык 60. Бросили 75. 75 > 60 -> Рост!
        let result = resolve_experience_roll(
            SkillRating::new(60),
            D100Roll::new(75),
            GrowthRoll::new(4),
            ExperienceBonus::new(0),
            MasteryTarget::new(80), // INTx5 (не используется, так как навык < 100)
        );
        assert_eq!(result, ExperienceRollResult::ImprovedBy(4));
    }

    #[test]
    fn test_normal_improvement_failure() {
        // Навык 60. Бросили 60. 60 НЕ БОЛЬШЕ 60 -> Провал.
        let result = resolve_experience_roll(
            SkillRating::new(60),
            D100Roll::new(60),
            GrowthRoll::new(6),
            ExperienceBonus::new(0),
            MasteryTarget::new(80),
        );
        assert_eq!(result, ExperienceRollResult::NoImprovement);
    }

    #[test]
    fn test_mastery_skill_100_percent() {
        // Навык 100. Target меняется на mastery_target (INTx5 = 75).
        // Бросили 80. 80 > 75 -> Рост!
        let result = resolve_experience_roll(
            SkillRating::new(100),
            D100Roll::new(80),
            GrowthRoll::new(2),
            ExperienceBonus::new(0),
            MasteryTarget::new(75), // INTx5
        );
        assert_eq!(result, ExperienceRollResult::ImprovedBy(2));
    }

    #[test]
    fn test_mastery_skill_101_percent() {
        // Навык 101. Target меняется на mastery_target (80).
        // Бросили 75. 75 НЕ БОЛЬШЕ 80 -> Провал.
        let result = resolve_experience_roll(
            SkillRating::new(101),
            D100Roll::new(75),
            GrowthRoll::new(3),
            ExperienceBonus::new(0),
            MasteryTarget::new(80), // INTx5
        );
        assert_eq!(result, ExperienceRollResult::NoImprovement);
    }

    #[test]
    fn test_experience_bonus_overflow_feature() {
        // Навык 99. Бросили 95 (провал без бонуса).
        // Бонус опыта 10 (INT 20 / 2). 95 + 10 = 105.
        // 105 > 99 -> Рост!
        let result = resolve_experience_roll(
            SkillRating::new(99),
            D100Roll::new(95),
            GrowthRoll::new(5),
            ExperienceBonus::new(10), // Бонус опыта
            MasteryTarget::new(80),
        );
        assert_eq!(result, ExperienceRollResult::ImprovedBy(5));
    }

    #[test]
    fn test_impossible_improvement_without_bonus() {
        // Навык 100, Mastery = 105 (Сверхразум с INT 21).
        // D100 максимум 100. Без бонусов он никогда не выкинет > 105.
        let result = resolve_experience_roll(
            SkillRating::new(100),
            D100Roll::new(100), // Максимальный бросок
            GrowthRoll::new(6),
            ExperienceBonus::new(0),
            MasteryTarget::new(105),
        );
        assert_eq!(result, ExperienceRollResult::NoImprovement);
    }
}
