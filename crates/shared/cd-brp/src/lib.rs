//! cd-brp: BRP UGE system implementation
//!
//! Core mechanics for Basic Roleplaying Universal Game Engine.

pub mod action_points;
pub mod anatomy;
pub mod characteristics;
pub mod combat;
pub mod dice;
pub mod encumbrance;
mod error;
pub mod rolls;
pub mod rules;
pub mod skills;

pub use action_points::ActionPoints;
pub use anatomy::{Anatomy, BodyPart, HitLocationType, Injury};
use bevy::app::{App, Plugin};
pub use characteristics::Characteristics;
pub use combat::{AttackResolution, CombatHitResult, EffectiveHit, WeaponDamage, WeaponSpecial};
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

pub struct BrpCorePlugin;

impl Plugin for BrpCorePlugin {
    fn build(&self, app: &mut App) {
        // Регистрируем компоненты BRP для инспектора
        app.register_type::<ActionPoints>()
            .register_type::<Encumbrance>()
            .register_type::<Anatomy>()
            .register_type::<Characteristics>();

        // ИНИЦИАЛИЗИРУЕМ ШИНЫ СООБЩЕНИЙ:
        app.init_resource::<bevy::ecs::message::Messages<anatomy::systems::damage::DamageMessage>>(
        );
        app.init_resource::<bevy::ecs::message::Messages<anatomy::AnatomyEvent>>();

        // Регистрируем саму систему урона (чтобы она начала работать в игровом цикле)
        app.add_systems(
            bevy::app::Update,
            anatomy::systems::damage::apply_damage_system,
        );
    }
}
