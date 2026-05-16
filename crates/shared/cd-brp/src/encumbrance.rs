use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{Changed, Or};
use bevy::ecs::system::Query;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::characteristics::Characteristics;
use crate::skills::{SkillCategory, SkillPercent};

// ============================================================================
// Константы балансировки (легко вынести в Resource/Config позже)
// ============================================================================

/// Штраф к MOV/Action Points за каждое очко перегруза
pub const MOV_PENALTY_PER_ENC: i32 = 1;
/// Штраф к навыкам в % за каждое очко перегруза
pub const SKILL_PENALTY_PER_ENC: i32 = 5;

// ============================================================================
/// Состояние перегруза. Используется для геймплейных флагов (анимации, блокировки действий)
/// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncumbranceLevel {
    #[default]
    None,
    Light,
    Medium,
    Heavy,
    Overburdened,
}

impl EncumbranceLevel {
    /// Определяет уровень по превышению лимита
    #[must_use]
    pub fn from_overage(overage: i32) -> Self {
        match overage {
            0 => Self::None,
            1..=2 => Self::Light,
            3..=5 => Self::Medium,
            6..=10 => Self::Heavy,
            _ => Self::Overburdened,
        }
    }

    /// Геймплейные флаги для Roguelike
    #[must_use]
    pub fn blocks_sprint(&self) -> bool {
        matches!(self, Self::Heavy | Self::Overburdened)
    }

    #[must_use]
    pub fn causes_passive_stamina_drain(&self) -> bool {
        matches!(self, Self::Overburdened)
    }
}

// ============================================================================
/// Компонент текущей нагрузки. Сериализуется и реплицируется по сети.
/// ============================================================================
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Component, Reflect,
)]
pub struct Encumbrance {
    pub current: i32,
}

impl Encumbrance {
    #[must_use]
    pub fn new(current: i32) -> Self {
        Self { current }
    }

    /// Максимальная грузоподъёмность по BRP UGE (стр. 137)
    /// (STR + CON) / 2 с округлением вверх
    #[must_use]
    pub fn max_enc(chars: &Characteristics) -> i32 {
        (chars.str + chars.con + 1) / 2
    }

    /// Вычисляет производные штрафы.
    /// В ECS предпочтительно вызывать не вручную, а через систему `update_encumbrance_penalties`.
    #[must_use]
    pub fn calculate_penalties(&self, chars: &Characteristics) -> EncumbrancePenalties {
        let max = Self::max_enc(chars);
        let overage = (self.current - max).max(0);

        EncumbrancePenalties {
            overage,
            mov_penalty: overage * MOV_PENALTY_PER_ENC,
            raw_skill_penalty: (overage * SKILL_PENALTY_PER_ENC) as i16,
            level: EncumbranceLevel::from_overage(overage),
        }
    }
}

// ============================================================================
/// Производный компонент штрафов. НЕ сериализуется, рассчитывается на сервере.
/// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub struct EncumbrancePenalties {
    /// На сколько очков превышен лимит
    pub overage: i32,
    /// Штраф к MOV / Action Points
    pub mov_penalty: i32,
    /// Сырой штраф в процентах (всегда >= 0). Применяется как вычитание.
    pub raw_skill_penalty: i16,
    /// Текущий уровень перегруза
    pub level: EncumbranceLevel,
}

impl EncumbrancePenalties {
    /// Возвращает модификатор для категории навыков.
    /// По BRP штрафуются только боевые, манипуляционные и физические навыки.
    #[must_use]
    pub fn skill_modifier_for(&self, category: SkillCategory) -> i16 {
        match category {
            SkillCategory::Combat | SkillCategory::Manipulation | SkillCategory::Physical => {
                -self.raw_skill_penalty
            }
            _ => 0, // Communication, Mental, Perception не страдают от веса
        }
    }

    /// Безопасно применяет штраф к шансу навыка с учётом clamp [1, 200]
    #[must_use]
    pub fn apply_to_skill(&self, base: SkillPercent, category: SkillCategory) -> SkillPercent {
        let modifier = self.skill_modifier_for(category);
        SkillPercent::new(base.get() + modifier)
    }

    #[must_use]
    pub fn is_none(&self) -> bool {
        self.overage == 0
    }
}

// ============================================================================
/// ECS Системы автоматического пересчёта
/// ============================================================================

/// Обновляет производные штрафы при изменении инвентаря или характеристик.
/// Запускается только при реальных изменениях (Changed<>).
pub fn update_encumbrance_penalties(
    mut query: Query<
        (&Encumbrance, &Characteristics, &mut EncumbrancePenalties),
        Or<(Changed<Encumbrance>, Changed<Characteristics>)>,
    >,
) {
    for (enc, chars, mut penalties) in query.iter_mut() {
        *penalties = enc.calculate_penalties(chars);
    }
}

/// Интегрирует штрафы перегруза в кэшированные шансы навыков.
/// Должен запускаться ПОСЛЕ `update_encumbrance_penalties` и `recalculate_skill_chances`
pub fn apply_encumbrance_to_cached_skills(
    penalties_query: Query<(Entity, &EncumbrancePenalties)>,
    mut skills_query: Query<(
        &mut crate::skills::CachedSkillChance,
        &crate::skills::SkillData,
    )>,
) {
    // Оптимизация: если ни у кого нет перегруза, пропускаем тяжёлую логику
    let has_active_penalties = penalties_query.iter().any(|(_, p)| p.overage > 0);
    if !has_active_penalties {
        return;
    }

    for (entity, penalties) in penalties_query.iter() {
        if let Ok((mut cached, skill_data)) = skills_query.get_mut(entity) {
            let modifier = penalties.skill_modifier_for(skill_data.category);
            cached.0 = SkillPercent::new(cached.0.get() + modifier);
        }
    }
}

// ============================================================================
/// Тесты
/// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::characteristics::Characteristics;

    #[test]
    fn test_max_enc_rounding() {
        // STR 14, CON 13 => (14+13+1)/2 = 14
        let chars = Characteristics::new(14, 13, 10, 10, 10, 10, 10, 10);
        assert_eq!(Encumbrance::max_enc(&chars), 14);

        // STR 15, CON 14 => (15+14+1)/2 = 15
        let chars_odd = Characteristics::new(15, 14, 10, 10, 10, 10, 10, 10);
        assert_eq!(Encumbrance::max_enc(&chars_odd), 15);

        // Минимальные значения
        let chars_min = Characteristics::new(1, 1, 10, 10, 10, 10, 10, 10);
        assert_eq!(Encumbrance::max_enc(&chars_min), 1);
    }

    #[test]
    fn test_encumbrance_level_thresholds() {
        assert_eq!(EncumbranceLevel::from_overage(0), EncumbranceLevel::None);
        assert_eq!(EncumbranceLevel::from_overage(1), EncumbranceLevel::Light);
        assert_eq!(EncumbranceLevel::from_overage(2), EncumbranceLevel::Light);
        assert_eq!(EncumbranceLevel::from_overage(3), EncumbranceLevel::Medium);
        assert_eq!(EncumbranceLevel::from_overage(5), EncumbranceLevel::Medium);
        assert_eq!(EncumbranceLevel::from_overage(6), EncumbranceLevel::Heavy);
        assert_eq!(EncumbranceLevel::from_overage(10), EncumbranceLevel::Heavy);
        assert_eq!(
            EncumbranceLevel::from_overage(11),
            EncumbranceLevel::Overburdened
        );
    }

    #[test]
    fn test_penalty_calculation_no_overload() {
        let chars = Characteristics::new(14, 13, 12, 10, 10, 10, 10, 10);
        let enc = Encumbrance::new(14);
        let penalties = enc.calculate_penalties(&chars);

        assert_eq!(penalties.overage, 0);
        assert_eq!(penalties.mov_penalty, 0);
        assert_eq!(penalties.raw_skill_penalty, 0);
        assert_eq!(penalties.level, EncumbranceLevel::None);
        assert!(penalties.is_none());
    }

    #[test]
    fn test_penalty_calculation_overloaded() {
        let chars = Characteristics::new(14, 13, 12, 10, 10, 10, 10, 10);
        // Текущий вес 17 => перегруз 3
        let enc = Encumbrance::new(17);
        let penalties = enc.calculate_penalties(&chars);

        assert_eq!(penalties.overage, 3);
        assert_eq!(penalties.mov_penalty, 3); // 3 * MOV_PENALTY_PER_ENC
        assert_eq!(penalties.raw_skill_penalty, 15); // 3 * SKILL_PENALTY_PER_ENC
        assert_eq!(penalties.level, EncumbranceLevel::Medium);
        assert!(!penalties.is_none());
    }

    #[test]
    fn test_skill_modifier_category_filter() {
        let penalties = EncumbrancePenalties {
            overage: 3,
            mov_penalty: 3,
            raw_skill_penalty: 15,
            level: EncumbranceLevel::Medium,
        };

        // Штрафуются
        assert_eq!(penalties.skill_modifier_for(SkillCategory::Combat), -15);
        assert_eq!(
            penalties.skill_modifier_for(SkillCategory::Manipulation),
            -15
        );
        assert_eq!(penalties.skill_modifier_for(SkillCategory::Physical), -15);

        // Не штрафуются
        assert_eq!(
            penalties.skill_modifier_for(SkillCategory::Communication),
            0
        );
        assert_eq!(penalties.skill_modifier_for(SkillCategory::Mental), 0);
        assert_eq!(penalties.skill_modifier_for(SkillCategory::Perception), 0);
    }

    #[test]
    fn test_apply_to_skill_clamping() {
        let penalties = EncumbrancePenalties {
            overage: 40, // Экстремальный перегруз для теста границ
            mov_penalty: 40,
            raw_skill_penalty: 200, // 40 * 5
            level: EncumbranceLevel::Overburdened,
        };

        // Навык 50% => 50 - 200 = -150 => clamp до MIN (1%)
        let base = SkillPercent::new(50);
        let result = penalties.apply_to_skill(base, SkillCategory::Combat);
        assert_eq!(result.get(), SkillPercent::MIN.get()); // 1

        // Навык 1% => 1 - 200 = -199 => clamp до MIN (1%)
        let base_low = SkillPercent::new(1);
        let result_low = penalties.apply_to_skill(base_low, SkillCategory::Combat);
        assert_eq!(result_low.get(), 1);
    }

    #[test]
    fn test_encumbrance_level_flags() {
        assert!(!EncumbranceLevel::None.blocks_sprint());
        assert!(!EncumbranceLevel::Light.blocks_sprint());
        assert!(!EncumbranceLevel::Medium.blocks_sprint());
        assert!(EncumbranceLevel::Heavy.blocks_sprint());
        assert!(EncumbranceLevel::Overburdened.blocks_sprint());

        assert!(!EncumbranceLevel::Medium.causes_passive_stamina_drain());
        assert!(EncumbranceLevel::Overburdened.causes_passive_stamina_drain());
    }
}
