//! Модуль вычисления модификаторов и финальных шансов навыков.

use crate::domain::chars::CharacteristicBlock;
use crate::math::BrpFractions;
use crate::types::{DifficultyModifier, GameSessionConfig, SkillCategory, SkillRating};
use crate::{D100_MAX, HUMAN_AVERAGE_STAT};

/// Пороговые значения рейтингов навыков (стр. 67)
pub mod skill_rating {
    pub const NOVICE_MAX: u16 = 5; // 00-05%
    pub const NEOPHYTE_MAX: u16 = 25; // 06-25%
    pub const AMATEUR_MAX: u16 = 50; // 26-50%
    pub const PROFESSIONAL_MAX: u16 = 75; // 51-75%
    pub const EXPERT_MAX: u16 = 90; // 76-90%
    pub const MASTER_MIN: u16 = 91; // 91%+
}

/// Применяет множитель сложности к рейтингу навыка (стр. 23-24).
#[must_use = "результат должен быть применён к навыку"]
pub fn apply_difficulty(rating: SkillRating, difficulty: DifficultyModifier) -> SkillRating {
    let val = rating.get();

    match difficulty {
        // Специальные значения — не участвуют в обычных расчётах
        DifficultyModifier::Automatic => SkillRating::new(u16::MAX),
        DifficultyModifier::Impossible => SkillRating::ZERO,

        // Мультипликативные модификаторы (стр. 23-24)
        DifficultyModifier::Easy => {
            // ×2, но не больше 100%
            SkillRating::new((val.saturating_mul(2)).min(D100_MAX))
        }
        DifficultyModifier::Average => rating,
        DifficultyModifier::Difficult => {
            // ×½ с округлением вверх
            SkillRating::new(val.half_ceil())
        }
        DifficultyModifier::Extreme => {
            // ×⅕ — стандартное значение для сложных задач в BRP
            SkillRating::new(val.saturating_mul(20).saturating_div(100))
        }
    }
}

#[inline(always)]
const fn primary_mod(stat: u16) -> i16 {
    stat as i16 - HUMAN_AVERAGE_STAT as i16
}

#[inline(always)]
const fn secondary_mod(stat: u16) -> i16 {
    let diff = stat as i16 - HUMAN_AVERAGE_STAT as i16;
    // Округление вверх для положительных, вниз для отрицательных
    if diff >= 0 { (diff + 1) / 2 } else { diff / 2 }
}

#[inline(always)]
const fn negative_mod(stat: u16) -> i16 {
    -(stat as i16 - HUMAN_AVERAGE_STAT as i16)
}

/// Вычисляет бонус категории навыков по классической сложной формуле (стр. 43).
#[inline]
pub const fn calculate_category_bonus(
    category: SkillCategory,
    chars: &CharacteristicBlock,
    config: &GameSessionConfig,
) -> i16 {
    if !config.use_skill_category_bonuses {
        return 0;
    }

    let str_v = chars.str.get();
    let con_v = chars.con.get();
    let siz_v = chars.siz.get();
    let int_v = chars.int.get();
    let pow_v = chars.pow.get();
    let dex_v = chars.dex.get();
    let cha_v = chars.cha.get();

    match category {
        SkillCategory::Combat => primary_mod(dex_v) + secondary_mod(int_v) + secondary_mod(str_v),
        SkillCategory::Communication => {
            primary_mod(int_v) + secondary_mod(pow_v) + secondary_mod(cha_v)
        }
        SkillCategory::Manipulation => {
            primary_mod(dex_v) + secondary_mod(int_v) + secondary_mod(str_v)
        }
        SkillCategory::Mental => {
            let mut base = primary_mod(int_v) + secondary_mod(pow_v);
            if let Some(edu) = chars.edu {
                base += secondary_mod(edu.get());
            }
            base
        }
        SkillCategory::Perception => {
            primary_mod(int_v) + secondary_mod(pow_v) + secondary_mod(con_v)
        }
        SkillCategory::Physical => {
            primary_mod(dex_v) + secondary_mod(str_v) + secondary_mod(con_v) + negative_mod(siz_v)
        }
    }
}

/// Вычисляет бонус категории по альтернативной, Упрощенной Формуле (стр. 44).
pub fn calculate_simple_category_bonus(
    category: SkillCategory,
    chars: &CharacteristicBlock,
) -> i16 {
    let primary_stat = match category {
        SkillCategory::Combat => chars.dex.get(),
        SkillCategory::Communication => chars.cha.get(),
        SkillCategory::Manipulation => chars.dex.get(),
        SkillCategory::Mental => chars.int.get(),
        SkillCategory::Perception => chars.pow.get(),
        SkillCategory::Physical => chars.str.get(),
    };

    // Просто делим пополам с округлением вверх
    primary_stat.half_ceil() as i16
}

/// Вычисляет финальный (Эффективный) шанс навыка перед броском кубика.
/// Строго соблюдает порядок операций из рулбука (Стр. 24 "Situational Modifiers").
pub fn calculate_effective_skill(
    nominal_rating: SkillRating,
    difficulty: DifficultyModifier,
    situational_modifiers_sum: i16, // Сумма всех +/- % (напр. погода, инструменты)
) -> SkillRating {
    // Крайние случаи: не модифицируются ситуативно
    if matches!(
        difficulty,
        DifficultyModifier::Automatic | DifficultyModifier::Impossible
    ) {
        return apply_difficulty(nominal_rating, difficulty);
    }

    // 1. Применяем сложность (умножение/деление)
    let rating_after_difficulty = apply_difficulty(nominal_rating, difficulty);

    // 2. Применяем ситуативные модификаторы (+20%, -50% и т.д.)
    let final_value = (rating_after_difficulty.get() as i16)
        .saturating_add(situational_modifiers_sum)
        .clamp(0, D100_MAX as i16) as u16;

    SkillRating::new(final_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_easy_caps_at_100() {
        let base = SkillRating::new(60);
        let result = apply_difficulty(base, DifficultyModifier::Easy);
        assert_eq!(result.get(), 100); // Не 120!
    }

    #[test]
    fn test_difficulty_extreme() {
        let base = SkillRating::new(75);
        let result = apply_difficulty(base, DifficultyModifier::Extreme);
        assert_eq!(result.get(), 15); // 75 / 5 = 15
    }

    #[test]
    fn test_effective_skill_with_situational_mods() {
        let base = SkillRating::new(50);
        let result = calculate_effective_skill(
            base,
            DifficultyModifier::Difficult, // 50 → 25
            30,                            // +30% за отличные инструменты
        );
        assert_eq!(result.get(), 55); // 25 + 30 = 55, не больше 100
    }

    #[test]
    fn test_automatic_bypasses_modifiers() {
        let base = SkillRating::new(10);
        let result = calculate_effective_skill(
            base,
            DifficultyModifier::Automatic,
            -50, // Даже огромный штраф не влияет
        );
        assert!(result.is_automatic());
    }
}
