//! Модуль вычисления модификаторов и финальных шансов навыков.

use crate::HUMAN_AVERAGE_STAT;
use crate::domain::chars::CharacteristicBlock;
use crate::math::BrpFractions;
use crate::types::{DifficultyModifier, GameSessionConfig, SkillCategory, SkillRating};

/// Применяет множитель сложности к рейтингу навыка (стр. 23-24).
#[must_use = "результат должен быть применён к навыку"]
pub fn apply_difficulty(rating: SkillRating, difficulty: DifficultyModifier) -> SkillRating {
    let val = rating.get();

    match difficulty {
        DifficultyModifier::Automatic => SkillRating::new(u16::MAX), // Никогда не проваливается
        DifficultyModifier::Easy => SkillRating::new(val.saturating_mul(2)),
        DifficultyModifier::Average => rating,
        DifficultyModifier::Difficult => SkillRating::new(val.half_ceil()),
        DifficultyModifier::Impossible => SkillRating::ZERO, // 0% шанс
    }
}

#[inline(always)]
const fn primary_mod(stat: u16) -> i16 {
    stat as i16 - HUMAN_AVERAGE_STAT as i16
}

#[inline(always)]
const fn secondary_mod(stat: u16) -> i16 {
    (stat as i16 - HUMAN_AVERAGE_STAT as i16) / 2
}

#[inline(always)]
const fn negative_mod(stat: u16) -> i16 {
    -(stat as i16 - HUMAN_AVERAGE_STAT as i16)
}

/// Вычисляет бонус категории навыков по классической сложной формуле (стр. 43).
#[inline]
pub fn calculate_category_bonus(
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
        .clamp(0, u16::MAX as i16) as u16;

    SkillRating::new(final_value)
}
