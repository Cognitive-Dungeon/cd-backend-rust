// crates/shared/cd-brp/src/rules/ranged.rs
//! Модуль дистанционного боя (Ranged Combat, стр. 60-65).

use crate::rules::modifiers::apply_difficulty;
use crate::types::{DifficultyModifier, RangeCategory, SkillRating, WeaponClass, WeaponPropulsion};
use crate::{CloudCover, Precipitation, Siz, Stat, WindForce, calculate_effective_skill};

/// Входящие данные для расчета шанса попадания выстрела/броска
pub struct RangedShotContext {
    pub base_skill: SkillRating,

    // --- Оружейные факторы ---
    pub range: RangeCategory,
    /// Количество полных раундов, потраченных на прицеливание (стр. 60). Максимум 3.
    pub aiming_rounds: u8,

    // --- Факторы цели ---
    /// SIZ цели (очень большие цели легче поразить, стр. 62)
    pub target_siz: Stat<Siz>,
    /// Двигается ли цель на скорости > половины своего MOV (стр. 62)
    pub target_moving: bool,
    /// Стрельба в свалку ближнего боя (стр. 61)
    pub shooting_into_melee: bool,

    // --- Окружающая среда (стр. 119-120) ---
    pub wind: WindForce,
    pub precipitation: Precipitation,
    pub cloud_cover: CloudCover,

    // --- Суммарный модификатор от состояний (Buffs/Debuffs) ---
    // Сюда сервер кладет сумму всех бонусов от крафта, баффов и штрафов (например, -20% за рану).
    pub situational_modifiers_sum: i16,
}

/// Вычисляет финальный шанс попадания при дистанционной атаке.
#[must_use]
pub fn calculate_ranged_chance(
    ctx: RangedShotContext,
    propulsion: WeaponPropulsion,
) -> SkillRating {
    let mut current_skill = ctx.base_skill;

    // Мы собираем мультипликаторы сложности (Easy/Difficult) в массив (или применяем каскадом).
    // В BRP несколько Difficult модификаторов ОБЫЧНО не складываются в 1/4 (это опционально),
    // но если есть Extreme или Impossible, они перекрывают всё.
    let mut final_difficulty = DifficultyModifier::Average;

    // Хелпер для наслаивания сложностей (Берем наихудший сценарий)
    let mut apply_worse_diff = |new_diff: DifficultyModifier| {
        // Упрощенная логика: Impossible перекрывает всё, Difficult перекрывает Average и т.д.
        if new_diff == DifficultyModifier::Impossible
            || final_difficulty == DifficultyModifier::Impossible
        {
            final_difficulty = DifficultyModifier::Impossible;
        } else if new_diff == DifficultyModifier::Extreme {
            final_difficulty = DifficultyModifier::Extreme;
        } else if new_diff == DifficultyModifier::Difficult
            && final_difficulty != DifficultyModifier::Extreme
        {
            final_difficulty = DifficultyModifier::Difficult;
        } else if new_diff == DifficultyModifier::Easy
            && final_difficulty == DifficultyModifier::Average
        {
            final_difficulty = DifficultyModifier::Easy;
        }
    };

    // 1. Прицеливание (Aiming, стр. 60)
    // В BRP UGE прицеливание обычно дает фиксированный +10% или +20% (зависит от опций),
    // либо делает бросок Easy. Будем придерживаться классического +10% (увеличиваем рейтинг).
    if ctx.aiming_rounds > 0 {
        // В BRP модификаторы шанса применяются до умножения сложности
        let aim_bonus = (ctx.aiming_rounds.min(3) as u16) * 10;
        current_skill = current_skill + aim_bonus;
    }

    // 2. Модификатор дистанции (стр. 60-61)

    // TODO: Модификатор дистанции приравнивает правила неявно
    // Например Double Base Range к Normal и далее, что может ввести в заблуждение.
    // Пересмотреть логоику или нейминг для соответствия рулбуку
    match ctx.range {
        // Point-Blank (В упор) -> Easy (x2 шанс)
        RangeCategory::PointBlank => apply_worse_diff(DifficultyModifier::Easy),
        // Base Range (В пределах базовой дистанции оружия) -> Average
        RangeCategory::BaseRange => apply_worse_diff(DifficultyModifier::Average),
        // Double Base Range -> Difficult (x1/2 шанс)
        RangeCategory::DoubleBaseRange => apply_worse_diff(DifficultyModifier::Difficult),
        // Свыше двойной дистанции стрельба невозможна
        RangeCategory::BeyondDoubleBaseRange => apply_worse_diff(DifficultyModifier::Impossible),
    };

    // 3. Размер цели (Target Size, стр. 62)
    // SIZ 30+ -> Easy (огромные цели). SIZ 5- -> Difficult (мелкие цели).
    if ctx.target_siz.get() >= 30 {
        apply_worse_diff(DifficultyModifier::Easy);
    } else if ctx.target_siz.get() <= 5 {
        apply_worse_diff(DifficultyModifier::Difficult);
    }

    // 4. Движение и Свалка (Target Movement & Melee, стр. 62)
    // "Target moves rapidly" -> Difficult
    if ctx.target_moving || ctx.shooting_into_melee {
        apply_worse_diff(DifficultyModifier::Difficult);
    }

    // 5. ВЛИЯНИЕ ПОГОДЫ (Стр. 119-120)
    // Ветер влияет ТОЛЬКО на MusclePropelled (стрелы, дротики)
    if propulsion == WeaponPropulsion::MusclePropelled {
        match ctx.wind {
            WindForce::Strong | WindForce::Severe => {
                apply_worse_diff(DifficultyModifier::Difficult)
            }
            WindForce::Windstorm | WindForce::Hurricane | WindForce::Tornado => {
                apply_worse_diff(DifficultyModifier::Impossible)
            }
            WindForce::Light => {}
        }
    }

    // Осадки и туман (Влияют на видимость)
    // Считаем, что если цель не в упор (PointBlank), плохая погода мешает прицелиться.
    if ctx.range != RangeCategory::PointBlank {
        if matches!(
            ctx.precipitation,
            Precipitation::Snow | Precipitation::Sleet | Precipitation::Hail
        ) {
            apply_worse_diff(DifficultyModifier::Difficult);
        }
        if matches!(ctx.cloud_cover, CloudCover::Severe | CloudCover::Complete) {
            // Полная темнота/туман
            apply_worse_diff(DifficultyModifier::Difficult);
        }
    }

    calculate_effective_skill(
        current_skill,
        final_difficulty,
        ctx.situational_modifiers_sum,
    )
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
