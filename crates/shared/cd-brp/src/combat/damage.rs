use super::types::{EffectiveHit, WeaponDamage, WeaponSpecial};
use crate::dice::{DamageModifier, Sign};
use rand::Rng;

/// Crushing: удваивает DM перед броском (стр. 150).
pub fn apply_crushing_modifier(dmg_mod: DamageModifier) -> DamageModifier {
    match dmg_mod.sign {
        Sign::Negative => DamageModifier::NONE,
        Sign::None => DamageModifier::new(Sign::Positive, 1, crate::dice::DiceType::D4),
        Sign::Positive => DamageModifier::new(Sign::Positive, dmg_mod.count * 2, dmg_mod.dice),
    }
}

/// Бросает урон оружия с учётом типа попадания и спецэффекта.
pub fn roll_weapon_damage<R: Rng + ?Sized>(
    weapon: &WeaponDamage,
    weapon_special: WeaponSpecial,
    hit_type: EffectiveHit,
    rng: &mut R,
) -> i32 {
    match hit_type {
        EffectiveHit::Critical => weapon.max_damage(),
        EffectiveHit::Special if weapon_special == WeaponSpecial::Impaling => {
            // Impale удваивает кубики оружия, DM прибавляется отдельно
            WeaponDamage::new(weapon.count * 2, weapon.dice, weapon.flat_bonus * 2).roll(rng)
        }
        _ => weapon.roll(rng),
    }
}
