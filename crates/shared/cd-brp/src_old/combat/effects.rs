use super::types::{CombatHitResult, WeaponSpecial};
use crate::resistance_chance;
use rand::Rng;
use rand::RngExt;

/// Спецэффекты, применяемые к телу цели при Special/Critical.
pub fn apply_body_specials<R: Rng + ?Sized>(
    weapon_special: WeaponSpecial,
    total_damage: i32,
    target_siz: i32,
    result: &mut CombatHitResult,
    rng: &mut R,
) {
    match weapon_special {
        WeaponSpecial::Bleeding => result.apply_bleeding = true,
        WeaponSpecial::Entangling => result.entangling = true,
        WeaponSpecial::Knockback => {
            let chance = resistance_chance(total_damage, target_siz);
            if rng.random_range(1..=100) <= chance {
                result.knockback_meters = (total_damage / 5).max(1);
            }
        }
        _ => {}
    }
}
