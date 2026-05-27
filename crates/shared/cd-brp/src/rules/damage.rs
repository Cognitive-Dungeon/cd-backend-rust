use serde::{Deserialize, Serialize};

use crate::rules::combat_matrix::TargetHitType;
use crate::types::{HitPoints, SpecialSuccessEffect};
use crate::{ArmorPoints, DamagePoints};

/// Финальный результат, готовый к отправке по сети в `CombatEffect` и `NarrativeEvent`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageApplication {
    // Фактический урон, нанесённый здоровью цели (после брони).
    pub actual_damage_taken: HitPoints,
    /// Сколько урона было поглощено бронёй.
    pub armor_mitigated: HitPoints,
    /// Специальный эффект оружия, если был активирован (Impale, Bleed и т.д.).
    pub special_triggered: Option<SpecialSuccessEffect>,
    /// Был ли это критический удар (игнорирует броню, макс. урон).
    pub is_critical: bool,
}

impl DamageApplication {
    #[inline]
    #[must_use]
    const fn new(
        actual: HitPoints,
        mitigated: HitPoints,
        special: Option<SpecialSuccessEffect>,
        is_crit: bool,
    ) -> Self {
        Self {
            actual_damage_taken: actual,
            armor_mitigated: mitigated,
            special_triggered: special,
            is_critical: is_crit,
        }
    }

    /// Проверка: был ли нанесён какой-либо урон здоровью.
    #[inline]
    #[must_use]
    pub const fn did_damage(&self) -> bool {
        self.actual_damage_taken.is_positive()
    }

    /// Проверка: была ли броня полностью пробита.
    #[inline]
    #[must_use]
    pub const fn armor_penetrated(&self) -> bool {
        !self.did_damage() && self.special_triggered.is_some()
    }
}

/// Вычисляет фактический урон по телу, учитывая броню, тип попадания и спецэффекты.
///
/// # Правила BRP (Стр. 49, 59):
/// 1. Damage Modifier НИКОГДА не удваивается при Impale. Он прибавляется в конце.
/// 2. При Critical берется МАКСИМАЛЬНЫЙ урон оружия, но Damage Modifier бросается и прибавляется как обычно.
/// 3. Урон не может быть отрицательным (если DM отрицательный и превышает урон оружия).
pub fn calculate_actual_damage(
    hit_type: TargetHitType,
    rolled_weapon_damage: DamagePoints, // Брошенный урон ТОЛЬКО от оружия
    damage_modifier_roll: i16,          // Бросок Damage Modifier (может быть отрицательным)
    max_weapon_damage: DamagePoints,    // Максимальный урон ТОЛЬКО от оружия (для критов)
    weapon_special: SpecialSuccessEffect, // Что делает оружие при Special (например, Impale)
    armor_value: ArmorPoints,           // Защита брони (уже вычисленная - рандомная или фикс)
) -> DamageApplication {
    use TargetHitType::*;

    // Вспомогательная функция: применяет множитель Impale ТОЛЬКО к урону оружия
    #[inline]
    const fn apply_impale(base_dmg: DamagePoints, special: SpecialSuccessEffect) -> DamagePoints {
        if matches!(special, SpecialSuccessEffect::Impaling) {
            base_dmg.saturating_mul(2)
        } else {
            base_dmg
        }
    }

    // Вспомогательная функция: безопасно прибавляет Damage Modifier (с защитой < 0)
    #[inline]
    const fn apply_dm(weapon_dmg: DamagePoints, dm: i16) -> DamagePoints {
        let dm_points = DamagePoints::new(dm);
        // TODO: Ждём const traits в stable чтобы упростить до (weapon_dmg + dm_points)
        let total = weapon_dmg.saturating_add(dm_points.get());
        total.clamp_to_min(DamagePoints::ZERO)
    }

    match hit_type {
        // ─────────────────────────────────────────────────────────────
        // Уворот: урона нет, броня не тратится
        // ─────────────────────────────────────────────────────────────
        Evaded => DamageApplication::new(HitPoints::ZERO, HitPoints::ZERO, None, false),

        // ─────────────────────────────────────────────────────────────
        // Обычное попадание: урон - броня
        // ─────────────────────────────────────────────────────────────
        Normal => {
            let total_dmg = apply_dm(rolled_weapon_damage, damage_modifier_roll);
            let armor = armor_value.get();

            let actual = total_dmg.saturating_sub(armor).get();
            let mitigated = armor.min(total_dmg.get()); // Явно: сколько броня реально поглотила

            DamageApplication::new(
                HitPoints::new(actual),
                HitPoints::new(mitigated),
                None,
                false,
            )
        }

        // ─────────────────────────────────────────────────────────────
        // Special: обычный урон (возможно ×2 для Impale) - броня + эффект
        // ─────────────────────────────────────────────────────────────
        Special => {
            // 1. Удваиваем ТОЛЬКО урон оружия (если Impale)
            let weapon_dmg = apply_impale(rolled_weapon_damage, weapon_special);
            // 2. Добавляем Damage Modifier (не удвоенный!)
            let total_dmg = apply_dm(weapon_dmg, damage_modifier_roll);

            let armor = armor_value.get();
            let actual = total_dmg.saturating_sub(armor_value.get()).get();
            let mitigated = armor.min(total_dmg.get());

            DamageApplication::new(
                HitPoints::new(actual),
                HitPoints::new(mitigated),
                Some(weapon_special),
                false,
            )
        }

        // ─────────────────────────────────────────────────────────────
        // Critical: макс. урон (возможно ×2 для Impale), броня игнорируется
        // ─────────────────────────────────────────────────────────────
        Critical => {
            // 1. Берем МАКСИМАЛЬНЫЙ урон оружия и удваиваем его (если Impale)
            let weapon_dmg = apply_impale(max_weapon_damage, weapon_special);
            // 2. Добавляем обычный, брошенный Damage Modifier
            let total_dmg = apply_dm(weapon_dmg, damage_modifier_roll).get();

            DamageApplication::new(
                HitPoints::new(total_dmg), // Весь урон идет в цель
                HitPoints::ZERO,           // Броня игнорируется
                Some(weapon_special),
                true,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_impale_brp_rules() {
        // Оружие 1D8 (бросили 4), DM +1D4 (бросили 3). Броня 5.
        // Impale: Удваиваем 4 -> 8. Прибавляем 3 -> 11 общего урона. Броня 5 -> 6 урона телу.
        let result = calculate_actual_damage(
            TargetHitType::Special,
            DamagePoints::new(4),
            3,
            DamagePoints::new(8),
            SpecialSuccessEffect::Impaling,
            ArmorPoints::new(5),
        );
        assert_eq!(result.actual_damage_taken.get(), 6);
        assert_eq!(result.armor_mitigated.get(), 5);
        assert_eq!(
            result.special_triggered,
            Some(SpecialSuccessEffect::Impaling)
        );
    }

    #[test]
    fn test_critical_ignores_armor_and_uses_max() {
        // Оружие 1D8 (макс 8), DM +1D4 (бросили 2). Броня 10.
        // Critical Impale: Удваиваем макс (8 * 2 = 16). Прибавляем DM (2) = 18. Броня игнорируется.
        let result = calculate_actual_damage(
            TargetHitType::Critical,
            DamagePoints::new(2), // Бросок самого оружия не важен, берется макс
            2,                    // DM бросается всегда!
            DamagePoints::new(8),
            SpecialSuccessEffect::Impaling,
            ArmorPoints::new(10),
        );
        assert_eq!(result.actual_damage_taken.get(), 18);
        assert_eq!(result.armor_mitigated, HitPoints::ZERO);
        assert!(result.is_critical);
    }

    #[test]
    fn test_negative_damage_modifier() {
        // Оружие 1D6 (бросили 2), DM -1D4 (бросили -4). Итог < 0.
        let result = calculate_actual_damage(
            TargetHitType::Normal,
            DamagePoints::new(2),
            -4,
            DamagePoints::new(6),
            SpecialSuccessEffect::None,
            ArmorPoints::ZERO,
        );
        assert_eq!(result.actual_damage_taken, HitPoints::ZERO); // Урон не уходит в минус
    }
}
