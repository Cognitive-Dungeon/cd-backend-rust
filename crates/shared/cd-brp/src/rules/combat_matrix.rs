//! Модуль разрешения боевых столкновений по матрице атаки/защиты BRP.
//! Zero-allocation: все вычисления на стеке, 100% `const fn`-совместимо.

use serde::{Deserialize, Serialize};

use crate::{SuccessLevel, types::HitPoints};

// ============================================================================
// КОНСТАНТЫ
// ============================================================================

/// Базовое значение для урона оружию при парировании (стр. 51)
mod damage_values {
    pub const CRIT_VS_SPECIAL: i16 = 2;
    pub const CRIT_VS_SUCCESS: i16 = 4;
    pub const PARRY_VS_CRIT: i16 = 1;
    pub const PARRY_VS_SPECIAL: i16 = 2;
}

// ============================================================================
// ТИПЫ
// ============================================================================

/// Тип попадания по самому защитнику (его телу).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TargetHitType {
    /// Защитник полностью избежал урона по телу (успешный Dodge или Parry).
    Evaded,
    /// Обычное попадание (броня работает штатно).
    Normal,
    /// Особое попадание (срабатывает эффект оружия: Impale, Bleed и т.д., броня работает).
    Special,
    /// Критическое попадание (макс. урон, обычно игнорирует броню).
    Critical,
}

impl Default for TargetHitType {
    #[inline]
    fn default() -> Self {
        Self::Evaded
    }
}

/// Итоговый результат столкновения (100% на стеке, без аллокаций).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Default)]
pub struct ExchangeOutcome {
    /// Эффект, примененный к самому персонажу-защитнику.
    pub target_hit: TargetHitType,
    /// Урон, который получило оружие или щит ЗАЩИТНИКА при парировании.
    pub defender_weapon_damage: HitPoints,
    /// Урон, который получило оружие АТАКУЮЩЕГО (если защитник парировал).
    pub attacker_weapon_damage: HitPoints,
}

impl ExchangeOutcome {
    /// Создаёт новый результат столкновения.
    #[inline]
    #[must_use]
    pub const fn new(hit: TargetHitType, defender_dmg: HitPoints, attacker_dmg: HitPoints) -> Self {
        Self {
            target_hit: hit,
            defender_weapon_damage: defender_dmg,
            attacker_weapon_damage: attacker_dmg,
        }
    }

    /// Быстрая проверка: был ли нанесён урон телу защитника.
    #[inline]
    #[must_use]
    pub const fn hit_body(&self) -> bool {
        !matches!(self.target_hit, TargetHitType::Evaded)
    }

    /// Быстрая проверка: было ли критическое попадание.
    #[inline]
    #[must_use]
    pub const fn is_critical(&self) -> bool {
        matches!(self.target_hit, TargetHitType::Critical)
    }
}

// ============================================================================
// ТРЕЙТ И РЕАЛИЗАЦИЯ
// ============================================================================

/// Разрешает столкновение в ближнем бою по таблице "Attack and Defence Matrix" (стр. 51).
pub trait MatrixResolver {
    #[must_use = "результат боя должен быть обработан (нанесён урон, проверена броня и т.д.)"]
    fn resolve_melee(
        attacker: SuccessLevel,
        defender: Option<SuccessLevel>,
        is_dodge: bool,
    ) -> ExchangeOutcome;
}

/// Стандартная реализация матрицы ближнего боя BRP.
///
/// # Алгоритм
/// 1. Если атакующий провалил бросок (`Failure`/`Fumble`) — атака автоматически отбита.
/// 2. Если защитник не защищался (`None`) — считается `Failure`.
/// 3. Урон оружию наносится **только** при парировании (не при увороте).
/// 4. Критические попадания обычно игнорируют броню (обрабатывается на уровне применения урона).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BrpCombatMatrix;

impl BrpCombatMatrix {
    /// Безопасная конвертация урона в `HitPoints` с защитой от отрицательных значений.
    #[inline]
    #[must_use]
    const fn saturating_item_damage(is_dodge: bool, raw_dmg: i16) -> HitPoints {
        if is_dodge {
            HitPoints::ZERO
        } else {
            let clamped = if raw_dmg < 0 { 0 } else { raw_dmg };
            HitPoints::new(clamped)
        }
    }
}

impl MatrixResolver for BrpCombatMatrix {
    #[inline]
    fn resolve_melee(
        attacker: SuccessLevel,
        defender: Option<SuccessLevel>,
        is_dodge: bool,
    ) -> ExchangeOutcome {
        use SuccessLevel::*;
        use TargetHitType::*;
        use damage_values::*;

        let def_level = defender.unwrap_or(Failure); // Если не защищался, считаем это провалом защиты

        // Макрос для уменьшения дублирования: (тип_попадания, урон_защитнику, урон_атакующему)
        macro_rules! outcome {
            ($hit:expr, $def_dmg:expr, $att_dmg:expr) => {
                ExchangeOutcome::new(
                    $hit,
                    Self::saturating_item_damage(is_dodge, $def_dmg),
                    Self::saturating_item_damage(is_dodge, $att_dmg),
                )
            };
        }

        match (attacker, def_level) {
            // ─────────────────────────────────────────────────────────────
            // Атакующий: CRITICAL SUCCESS
            // ─────────────────────────────────────────────────────────────
            (CriticalSuccess, CriticalSuccess) => outcome!(Evaded, 0, 0),
            (CriticalSuccess, SpecialSuccess) => outcome!(Normal, CRIT_VS_SPECIAL, 0),
            (CriticalSuccess, Success) => outcome!(Special, CRIT_VS_SUCCESS, 0),
            (CriticalSuccess, Failure | Fumble) => outcome!(Critical, 0, 0),

            // ─────────────────────────────────────────────────────────────
            // Атакующий: SPECIAL SUCCESS
            // ─────────────────────────────────────────────────────────────
            (SpecialSuccess, CriticalSuccess) => outcome!(Evaded, 0, PARRY_VS_CRIT),
            (SpecialSuccess, SpecialSuccess) => outcome!(Evaded, 0, 0),
            (SpecialSuccess, Success) => outcome!(Normal, PARRY_VS_SPECIAL, 0),
            (SpecialSuccess, Failure | Fumble) => outcome!(Special, 0, 0),

            // ─────────────────────────────────────────────────────────────
            // Атакующий: SUCCESS
            // ─────────────────────────────────────────────────────────────
            (Success, CriticalSuccess) => outcome!(Evaded, 0, PARRY_VS_SPECIAL),
            (Success, SpecialSuccess) => outcome!(Evaded, 0, PARRY_VS_CRIT),
            (Success, Success) => outcome!(Evaded, 0, 0),
            (Success, Failure | Fumble) => outcome!(Normal, 0, 0),

            // ─────────────────────────────────────────────────────────────
            // Атакующий: FAILURE / FUMBLE — атака автоматически провалена
            // ─────────────────────────────────────────────────────────────
            (Failure | Fumble, _) => outcome!(Evaded, 0, 0),
        }
    }
}

// ============================================================================
// ТЕСТЫ
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_vs_failure_is_critical_hit() {
        let result = BrpCombatMatrix::resolve_melee(
            SuccessLevel::CriticalSuccess,
            Some(SuccessLevel::Failure),
            false,
        );
        assert_eq!(result.target_hit, TargetHitType::Critical);
        assert_eq!(result.defender_weapon_damage, HitPoints::ZERO);
        assert_eq!(result.attacker_weapon_damage, HitPoints::ZERO);
    }

    #[test]
    fn test_dodge_never_damages_weapons() {
        // Даже если по правилам парирования должен быть урон оружию,
        // при увороте (Dodge) урон по предметам не наносится.
        let result = BrpCombatMatrix::resolve_melee(
            SuccessLevel::SpecialSuccess,
            Some(SuccessLevel::CriticalSuccess),
            true, // Dodge!
        );
        assert_eq!(result.target_hit, TargetHitType::Evaded);
        assert_eq!(result.defender_weapon_damage, HitPoints::ZERO);
        assert_eq!(result.attacker_weapon_damage, HitPoints::ZERO);
    }

    #[test]
    fn test_no_defense_treated_as_failure() {
        let result = BrpCombatMatrix::resolve_melee(
            SuccessLevel::Success,
            None, // защитник не пытался защищаться
            false,
        );
        assert_eq!(result.target_hit, TargetHitType::Normal);
    }

    #[test]
    fn test_fumble_always_evades() {
        let result = BrpCombatMatrix::resolve_melee(
            SuccessLevel::Fumble,
            Some(SuccessLevel::Failure),
            false,
        );
        assert_eq!(result.target_hit, TargetHitType::Evaded);
        assert_eq!(result.defender_weapon_damage, HitPoints::ZERO);
        assert_eq!(result.attacker_weapon_damage, HitPoints::ZERO);
    }

    #[test]
    fn test_crit_vs_special_parry_damage() {
        let result = BrpCombatMatrix::resolve_melee(
            SuccessLevel::CriticalSuccess,
            Some(SuccessLevel::SpecialSuccess),
            false, // Parry
        );
        assert_eq!(result.target_hit, TargetHitType::Normal);
        assert_eq!(result.defender_weapon_damage.get(), 2); // CRIT_VS_SPECIAL
        assert_eq!(result.attacker_weapon_damage, HitPoints::ZERO);
    }

    #[test]
    fn test_success_vs_success_is_evaded() {
        let result = BrpCombatMatrix::resolve_melee(
            SuccessLevel::Success,
            Some(SuccessLevel::Success),
            false,
        );
        assert_eq!(result.target_hit, TargetHitType::Evaded);
        assert_eq!(result.defender_weapon_damage, HitPoints::ZERO);
        assert_eq!(result.attacker_weapon_damage, HitPoints::ZERO);
    }

    #[test]
    fn test_helper_methods() {
        let hit = ExchangeOutcome::new(TargetHitType::Critical, HitPoints::ZERO, HitPoints::ZERO);
        assert!(hit.hit_body());
        assert!(hit.is_critical());

        let evade = ExchangeOutcome::new(TargetHitType::Evaded, HitPoints::ZERO, HitPoints::ZERO);
        assert!(!evade.hit_body());
        assert!(!evade.is_critical());
    }

    #[test]
    fn test_negative_damage_clamped() {
        // Проверяем, что даже если в будущем логика изменится и передаст отрицательный урон,
        // он будет безопасно обработан.
        let dmg = BrpCombatMatrix::saturating_item_damage(false, -5);
        assert_eq!(dmg, HitPoints::ZERO);
    }
}
