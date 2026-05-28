//! Модуль правил защиты в бою (Defense & Multiple Actions, стр. 55).

use crate::{
    DEFAULT_MULTIPLE_ACTION_PENALTY, HitPoints, SkillModifier, SuccessLevel,
    types::{SkillRating, WeaponClass},
};

/// Контекст для вычисления финального шанса защиты (Dodge или Parry).
pub struct DefenseRatingContext {
    pub base_skill: SkillRating,
    /// Сколько попыток защиты (Уклонений + Парирований) персонаж УЖЕ сделал в этом раунде.
    pub previous_defenses_this_round: u8,
    /// Сумма всех магических или ситуативных баффов/дебаффов.
    pub situational_modifier: SkillModifier,
}

/// Вычисляет шанс на успешное уклонение (Dodge) или парирование (Parry)
/// с учетом усталости от множественных атак в течение одного раунда.
#[must_use]
pub fn calculate_defense_chance(ctx: DefenseRatingContext) -> SkillRating {
    // 1. Считаем кумулятивный штраф (каждая защита после первой дает -30%)
    let penalty = ctx.previous_defenses_this_round as u16 * DEFAULT_MULTIPLE_ACTION_PENALTY;

    // 2. Вычитаем штраф из базового навыка
    let penalized_skill = ctx.base_skill - penalty;

    // 3. Применяем ситуативные модификаторы (с защитой от ухода в минус)
    penalized_skill + ctx.situational_modifier
}

/// Результат тактических эффектов защиты (Опциональные правила).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenseTacticalEffect {
    None,
    /// Стр. 51: Критическая защита против обычного успеха открывает врага для контратаки (Riposte).
    CounterAttackOpportunity,
    /// Фамбл на защите означает, что персонаж теряет равновесие, роняет оружие или получает двойной урон.
    DefenderStumbles,
}

/// Анализирует уровни успеха атаки и защиты, чтобы выдать тактические последствия.
/// Вызывается ПОСЛЕ основной матрицы `BrpCombatMatrix::resolve_melee`.
#[must_use]
pub fn resolve_defense_tactics(
    attack_level: SuccessLevel,
    defense_level: SuccessLevel,
) -> DefenseTacticalEffect {
    use SuccessLevel::*;

    // Критический провал защиты - всегда плохо
    if defense_level == Fumble {
        return DefenseTacticalEffect::DefenderStumbles;
    }

    // Блестящая защита против неуклюжей атаки
    if defense_level == CriticalSuccess && (attack_level == Success || attack_level == Failure) {
        return DefenseTacticalEffect::CounterAttackOpportunity;
    }

    DefenseTacticalEffect::None
}

/// Проверяет, можно ли использовать данное оружие (или щит) для защиты от другой атаки
/// в рамках одного Strike Rank (одновременные атаки).
///
/// В BRP обычно можно защититься от любого количества атак,
/// но есть нюансы с двуручным оружием и щитами (на усмотрение модулей).
/// Эта функция — задел на будущее (например, если щит был разрушен в этом раунде).
#[must_use]
pub const fn can_parry_with(weapon_class: WeaponClass, weapon_hp: HitPoints) -> bool {
    // Если оружие сломано (HP == 0), им нельзя парировать
    if weapon_hp.get() == 0 {
        return false;
    }

    // Некоторые виды оружия (например, метательное вроде гранаты или лассо)
    // не подходят для парирования.
    !matches!(
        weapon_class,
        WeaponClass::Grenade | WeaponClass::Explosive | WeaponClass::Bow | WeaponClass::Crossbow
    )
}

#[cfg(test)]
mod tests {
    use crate::IncomingAttackType;

    use super::*;

    #[test]
    fn test_parry_legality() {
        assert!(IncomingAttackType::Melee.is_parry_legal(WeaponClass::Sword));
        // Стрелу отбить мечом нельзя
        assert!(!IncomingAttackType::ThrownOrArrow.is_parry_legal(WeaponClass::Sword));
        // Стрелу отбить щитом можно
        assert!(IncomingAttackType::ThrownOrArrow.is_parry_legal(WeaponClass::Sword));
        // Пулю отбить щитом нельзя
        assert!(!IncomingAttackType::FirearmOrEnergy.is_parry_legal(WeaponClass::Shield));
    }

    #[test]
    fn test_multiple_defense_penalty() {
        let ctx = DefenseRatingContext {
            base_skill: SkillRating::new(80),
            previous_defenses_this_round: 2, // Это 3-я защита в раунде (-60%)
            situational_modifier: SkillModifier::ZERO,
        };

        // 80 - 60 = 20
        assert_eq!(calculate_defense_chance(ctx).get(), 20);
    }
}
