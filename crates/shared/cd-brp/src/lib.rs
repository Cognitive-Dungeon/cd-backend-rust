//! cd-brp: BRP UGE system implementation
//!
//! Core mechanics for Basic Roleplaying Universal Game Engine.

pub mod action_points;
pub mod anatomy;
pub mod characteristics;
pub mod dice;
pub mod encumbrance;
mod error;
pub mod rolls;
pub mod skills;

pub use action_points::ActionPoints;
pub use anatomy::{Anatomy, BodyPart, HitLocationType, Injury};
pub use characteristics::Characteristics;
pub use encumbrance::{Encumbrance, EncumbrancePenalties};
pub use error::{BrpError, BrpResult};
pub use rolls::{SuccessLevel, resistance_chance};

/// Версия крейта для проверки совместимости
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api() {
        let chars = Characteristics::default();
        let _hp = chars.max_hit_points();
        let _mod = chars.damage_modifier();
        let _xp = chars.experience_bonus();

        let _roll = dice::roll_modifier(_mod, &mut rand::rng());
    }
}
