//! Модуль правил Нагрузки (Encumbrance, стр. 31-33).

use crate::{Characteristic, Con, SkillType, Stat, Str, math::frac_u16, types::EncumbrancePoints};

/// Вычисляет максимальную нагрузку (Max ENC), которую персонаж может нести без штрафов.
/// По умолчанию это STR.
#[must_use]
pub const fn calculate_max_enc(str_stat: Stat<Str>) -> EncumbrancePoints {
    EncumbrancePoints::from_stat(str_stat)
}

/// Вычисляет предел нагрузки для длительных переходов (prolonged maneuvers).
/// Равен усредненному значению STR и CON с округлением вверх.
#[must_use]
pub const fn calculate_prolonged_max_enc(
    str_stat: Stat<Str>,
    con_stat: Stat<Con>,
) -> EncumbrancePoints {
    let sum = str_stat.get().saturating_add(con_stat.get());
    EncumbrancePoints::from_u16(frac_u16::half_ceil(sum))
}

/// Контейнер со всеми активными штрафами от текущей нагрузки.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncumbrancePenalties {
    /// Количество ENC сверх максимума
    pub excess: EncumbrancePoints,
}

impl EncumbrancePenalties {
    /// Создает расчет штрафов на основе текущей и максимальной нагрузки.
    #[must_use]
    pub const fn new(max: EncumbrancePoints, current: EncumbrancePoints) -> Self {
        Self {
            excess: current.saturating_sub(max.get()),
        }
    }

    /// Штраф к скорости передвижения (MOV). "-1 to Movement (MOV)" за каждый лишний ENC.
    #[must_use]
    pub const fn mov_penalty(&self) -> u16 {
        self.excess.get() as u16
    }

    /// Штраф к навыкам в процентах. "-5% to all..." за каждый лишний ENC.
    #[must_use]
    pub const fn skill_penalty_percent(&self) -> u16 {
        self.excess.get().saturating_mul(5) as u16
    }

    /// Потеря очков усталости. "loses 1 fatigue point per turn per additional ENC".
    #[must_use]
    pub const fn fatigue_drain_per_turn(&self) -> u16 {
        self.excess.get() as u16
    }

    /// Проверяет, применяется ли штраф `skill_penalty_percent` к конкретному навыку.
    /// Строго по тексту: "-5% to all Agility, Manipulation, Stealth, Dodge, and weapon skills".
    #[must_use]
    pub const fn applies_to_skill(&self, skill: &SkillType) -> bool {
        if self.excess.get() == 0 {
            return false;
        }

        // Если навык зависит от Ловкости или Силы — он получает штраф.
        matches!(
            skill.primary_characteristic(),
            Characteristic::Dex | Characteristic::Str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WeaponClass;

    #[test]
    fn test_encumbrance_penalties_math() {
        // STR 10, несем 13 ENC. Превышение = 3.
        let max_enc = calculate_max_enc(Stat::<Str>::new(10));
        let current = EncumbrancePoints::new(13);
        let penalties = EncumbrancePenalties::new(max_enc, current);

        assert_eq!(penalties.excess.get(), 3);
        assert_eq!(penalties.mov_penalty(), 3); // -3 MOV
        assert_eq!(penalties.skill_penalty_percent(), 15); // -15%
        assert_eq!(penalties.fatigue_drain_per_turn(), 3); // -3 FP в ход
    }

    #[test]
    fn test_penalty_applies_to_correct_skills() {
        let penalties = EncumbrancePenalties {
            excess: EncumbrancePoints::new(1),
        };

        // Применяется:
        assert!(penalties.applies_to_skill(&SkillType::Dodge));
        assert!(penalties.applies_to_skill(&SkillType::Stealth));
        assert!(penalties.applies_to_skill(&SkillType::WeaponAttack(
            crate::WeaponSkillCategory::Melee,
            WeaponClass::Sword
        )));
        assert!(penalties.applies_to_skill(&SkillType::FineManipulation)); // Из категории Manipulation

        // НЕ применяется:
        assert!(!penalties.applies_to_skill(&SkillType::Listen)); // Perception
        assert!(!penalties.applies_to_skill(&SkillType::Persuade)); // Communication
    }

    #[test]
    fn test_no_penalties_if_under_limit() {
        let max_enc = calculate_max_enc(Stat::<Str>::new(10));
        let current = EncumbrancePoints::new(10);
        let penalties = EncumbrancePenalties::new(max_enc, current);

        assert_eq!(penalties.skill_penalty_percent(), 0);
        assert!(!penalties.applies_to_skill(&SkillType::Dodge));
    }
}
