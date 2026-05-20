mod damage;
mod effects;
pub mod fumbles;
mod matrix;
mod parry;
mod types;

pub use types::*;

use crate::dice::DamageModifier;
use crate::rolls::SuccessLevel;
use rand::Rng;

/// Декларативный пайплайн расчёта боя.
/// Сохраняет оригинальную сигнатуру для обратной совместимости.
pub fn resolve_attack<R: Rng + ?Sized>(
    atk_level: SuccessLevel,
    def_level: Option<SuccessLevel>,
    weapon: &WeaponDamage,
    weapon_special: WeaponSpecial,
    mut dmg_mod: DamageModifier,
    target_siz: i32,
    parry_item_hp: Option<i32>,
    rng: &mut R,
) -> CombatHitResult {
    let mut result = CombatHitResult::default();

    // 1. Ранний выход при промахе/фамбле
    if atk_level == SuccessLevel::Fumble {
        result.attacker_fumbled = true;
    }
    if def_level == Some(SuccessLevel::Fumble) {
        result.defender_fumbled = true;
    }

    if matches!(atk_level, SuccessLevel::Failure | SuccessLevel::Fumble) {
        return result;
    }

    // 2. Разрешение матрицы (декларативный LUT)
    let resolution = matrix::resolve(atk_level, def_level);
    let hit_type = resolution.hit;

    let is_special_or_crit = matches!(atk_level, SuccessLevel::Special | SuccessLevel::Critical);

    // 3. Crushing модификатор DM (удваивается ПЕРЕД броском)
    if is_special_or_crit && weapon_special == WeaponSpecial::Crushing {
        result.stun_check = true;
        dmg_mod = damage::apply_crushing_modifier(dmg_mod);
    }

    // 4. Бросок урона
    let weapon_roll = damage::roll_weapon_damage(weapon, weapon_special, hit_type, rng);
    let dm_roll = crate::dice::roll_modifier(dmg_mod, rng);
    let total_damage = (weapon_roll + dm_roll).max(0);

    // 5. Распределение урона между целью и предметом парирования
    let parry = if is_special_or_crit
        && weapon_special == WeaponSpecial::Crushing
        && let Some(php) = parry_item_hp
    {
        // Crushing требует Resistance Roll против HP предмета
        parry::handle_crushing_parry(total_damage, resolution.defense_item_damage, php, rng)
    } else {
        parry::handle_standard_parry(
            total_damage,
            hit_type,
            resolution.defense_item_damage,
            parry_item_hp,
        )
    };

    result.target_damage = parry.target_damage;
    result.parry_item_damage = parry.item_damage;

    // Уничтожен ли предмет парирования?
    if let Some(php) = parry_item_hp {
        result.parry_item_destroyed = result.parry_item_damage >= php;
    }

    // До тела ничего не дошло — эффекты на тело не накладываем
    if result.target_damage == 0 {
        return result;
    }

    // 6. Флаг игнорирования брони при крите
    if hit_type == EffectiveHit::Critical {
        result.ignores_armor = true;
    }

    // 7. Спецэффекты на тело цели
    if is_special_or_crit {
        effects::apply_body_specials(weapon_special, total_damage, target_siz, &mut result, rng);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::DiceType;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn test_matrix_logic() {
        let res = matrix::resolve(SuccessLevel::Critical, Some(SuccessLevel::Success));
        assert_eq!(res.hit, EffectiveHit::Special);
        assert_eq!(res.defense_item_damage, 4);

        let res = matrix::resolve(SuccessLevel::Special, Some(SuccessLevel::Critical));
        assert_eq!(res.hit, EffectiveHit::MissOrBlocked);
        assert_eq!(res.defense_item_damage, 1);
    }

    #[test]
    fn test_critical_damage_ignores_armor() {
        let mut rng = SmallRng::seed_from_u64(42);
        let weapon = WeaponDamage::new(1, DiceType::D6, 1);
        let dm = DamageModifier::NONE;

        let result = resolve_attack(
            SuccessLevel::Critical,
            None,
            &weapon,
            WeaponSpecial::Bleeding,
            dm,
            10,
            None,
            &mut rng,
        );

        assert_eq!(result.target_damage, 7);
        assert!(result.ignores_armor);
        assert!(result.apply_bleeding);
    }

    #[test]
    fn test_impaling_special() {
        let mut rng = SmallRng::seed_from_u64(123);
        let weapon = WeaponDamage::new(1, DiceType::D6, 1);
        let dm = DamageModifier::NONE;

        let result = resolve_attack(
            SuccessLevel::Special,
            None,
            &weapon,
            WeaponSpecial::Impaling,
            dm,
            10,
            None,
            &mut rng,
        );

        assert!((4..=14).contains(&result.target_damage));
        assert!(!result.ignores_armor);
    }

    #[test]
    fn test_crushing_special_dm_doubling() {
        let mut rng = SmallRng::seed_from_u64(777);
        let weapon = WeaponDamage::new(1, DiceType::D8, 0);
        let base_dm = DamageModifier::new(crate::dice::Sign::Positive, 1, DiceType::D4);

        let result = resolve_attack(
            SuccessLevel::Special,
            None,
            &weapon,
            WeaponSpecial::Crushing,
            base_dm,
            10,
            None,
            &mut rng,
        );

        assert!(result.stun_check);
        assert!((3..=16).contains(&result.target_damage));
    }

    #[test]
    fn test_crushing_parry_attacker_wins() {
        let mut rng = SmallRng::seed_from_u64(1);
        let weapon = WeaponDamage::new(1, DiceType::D8, 10);
        let parry_hp = 5;

        let result = resolve_attack(
            SuccessLevel::Special,
            Some(SuccessLevel::Success),
            &weapon,
            WeaponSpecial::Crushing,
            DamageModifier::NONE,
            10,
            Some(parry_hp),
            &mut rng,
        );

        assert!(result.parry_item_damage > parry_hp);
        assert!(result.parry_item_destroyed);
        assert!(result.target_damage > 0);
        assert!(result.stun_check);
    }

    #[test]
    fn test_knockback_resolution() {
        let mut rng = SmallRng::seed_from_u64(999);
        let weapon = WeaponDamage::new(1, DiceType::D6, 20);
        let target_siz = 5;

        let result = resolve_attack(
            SuccessLevel::Special,
            None,
            &weapon,
            WeaponSpecial::Knockback,
            DamageModifier::NONE,
            target_siz,
            None,
            &mut rng,
        );

        assert!(result.knockback_meters > 0);
    }

    #[test]
    fn test_apply_crushing_modifier_doubles_positive() {
        let dm = DamageModifier::new(crate::dice::Sign::Positive, 1, DiceType::D6);
        let out = damage::apply_crushing_modifier(dm);
        assert_eq!(out.count, 2);
        assert_eq!(out.dice, DiceType::D6);
        assert_eq!(out.sign, crate::dice::Sign::Positive);
    }

    #[test]
    fn test_apply_crushing_modifier_negative_becomes_none() {
        let dm = DamageModifier::new(crate::dice::Sign::Negative, 1, DiceType::D4);
        let out = damage::apply_crushing_modifier(dm);
        assert_eq!(out, DamageModifier::NONE);
    }

    #[test]
    fn test_handle_standard_parry_no_item() {
        let p = parry::handle_standard_parry(10, EffectiveHit::Normal, 2, None);
        assert_eq!(p.item_damage, 0);
        assert_eq!(p.target_damage, 10);
    }

    #[test]
    fn test_handle_standard_parry_blocked() {
        let p = parry::handle_standard_parry(10, EffectiveHit::MissOrBlocked, 1, Some(8));
        assert_eq!(p.target_damage, 0);
        assert_eq!(p.item_damage, 1);
    }

    #[test]
    fn test_crushing_parry_item_wins_resistance() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        // Урон 10, HP щита 8 → шанс успеха сопротивления ~60%
        // Фиксируем RNG так, чтобы бросок 1..100 <= chance
        let mut rng = SmallRng::seed_from_u64(42);
        let out = parry::handle_crushing_parry(10, 2, 8, &mut rng);

        assert_eq!(out.target_damage, 0); // Цель в безопасности
        assert_eq!(out.item_damage, 2); // Урон по Матрице, а не 10
    }

    #[test]
    fn test_crushing_parry_item_loses_resistance() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        // Урон 15, HP щита 5 → шанс успеха ~20%
        // Подбираем сид, где бросок > chance (предмет ломается)
        let mut rng = SmallRng::seed_from_u64(99);
        let out = parry::handle_crushing_parry(15, 2, 5, &mut rng);

        assert_eq!(out.item_damage, 15); // Предмет принял полный удар
        assert_eq!(out.target_damage, 10); // (15 - 5) пробивает в цель
    }
}
