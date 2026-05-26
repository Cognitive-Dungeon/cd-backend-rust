// crates/shared/cd-brp/src/rules/ranged.rs
//! Модуль дистанционного боя (Ranged Combat, стр. 60-65).

use crate::rules::modifiers::apply_difficulty;
use crate::types::{DifficultyModifier, RangeCategory, SkillRating, WeaponClass, WeaponPropulsion};
use crate::{Siz, Stat};

/// Входящие данные для расчета шанса попадания выстрела/броска
pub struct RangedShotContext {
    pub base_skill: SkillRating,
    pub range: RangeCategory,
    /// Количество полных раундов, потраченных на прицеливание (стр. 60). Максимум 3.
    pub aiming_rounds: u8,
    /// SIZ цели (очень большие цели легче поразить, стр. 62)
    pub target_siz: Stat<Siz>,
    /// Двигается ли цель на скорости > половины своего MOV (стр. 62)
    pub target_moving: bool,
    /// Стрельба в свалку ближнего боя (стр. 61)
    pub shooting_into_melee: bool,
}

/// Вычисляет финальный шанс попадания при дистанционной атаке.
#[must_use]
pub fn calculate_ranged_chance(ctx: RangedShotContext) -> SkillRating {
    let mut current_skill = ctx.base_skill;

    // 1. Прицеливание (Aiming, стр. 60)
    // В BRP UGE прицеливание обычно дает фиксированный +10% или +20% (зависит от опций),
    // либо делает бросок Easy. Будем придерживаться классического +10% (увеличиваем рейтинг).
    if ctx.aiming_rounds > 0 {
        // В BRP модификаторы шанса применяются до умножения сложности
        let aim_bonus = (ctx.aiming_rounds.min(3) as u16) * 10;
        current_skill = current_skill.saturating_add(aim_bonus);
    }

    // 2. Модификатор дистанции (стр. 60-61)

    // TODO: Модификатор дистанции приравнивает правила неявно
    // Например Double Base Range к Normal и далее, что может ввести в заблуждение.
    // Пересмотреть логоику или нейминг для соответствия рулбуку
    let range_diff = match ctx.range {
        // Point-Blank (В упор) -> Easy (x2 шанс)
        RangeCategory::PointBlank => DifficultyModifier::Easy,
        // Base Range (В пределах базовой дистанции оружия) -> Average
        RangeCategory::BaseRange => DifficultyModifier::Average,
        // Double Base Range -> Difficult (x1/2 шанс)
        RangeCategory::DoubleBaseRange => DifficultyModifier::Difficult,
        // Свыше двойной дистанции стрельба невозможна
        RangeCategory::BeyondDoubleBaseRange => DifficultyModifier::Impossible,
    };
    current_skill = apply_difficulty(current_skill, range_diff);

    // 3. Размер цели (Target Size, стр. 62)
    // SIZ 30+ -> Easy (огромные цели). SIZ 5- -> Difficult (мелкие цели).
    if ctx.target_siz.get() >= 30 {
        current_skill = apply_difficulty(current_skill, DifficultyModifier::Easy);
    } else if ctx.target_siz.get() <= 5 {
        current_skill = apply_difficulty(current_skill, DifficultyModifier::Difficult);
    }

    // 4. Движение цели (Target Movement, стр. 62)
    // "Target moves rapidly" -> Difficult
    if ctx.target_moving {
        current_skill = apply_difficulty(current_skill, DifficultyModifier::Difficult);
    }

    // 5. Стрельба в ближний бой (Shooting into Melee, стр. 61)
    // "Shooting into Melee" -> Difficult. При провале есть шанс попасть в союзника.
    if ctx.shooting_into_melee {
        current_skill = apply_difficulty(current_skill, DifficultyModifier::Difficult);
    }

    current_skill
}

/// Определяет, имеет ли право защитник парировать дистанционную атаку (Стр. 64-65).
/// Огнестрел нельзя парировать ничем (кроме магических щитов).
/// Стрелы можно отбить только щитом.
#[must_use]
pub const fn can_parry_missile(
    attack_propulsion: WeaponPropulsion,
    defender_weapon_class: WeaponClass,
) -> bool {
    match attack_propulsion {
        // Огнестрел, лазеры (SelfPropelled) невозможно парировать обычным оружием.
        WeaponPropulsion::SelfPropelled => false,

        // Стрелы, дротики, копья (MusclePropelled)
        WeaponPropulsion::MusclePropelled => {
            // Парировать стрелы можно ТОЛЬКО щитом (Shield). Мечом отбить стрелу нельзя (без суперсил).
            matches!(defender_weapon_class, WeaponClass::Shield)
        }

        // Оружие ближнего боя всегда можно попытаться парировать.
        WeaponPropulsion::Melee => true,
    }
}
