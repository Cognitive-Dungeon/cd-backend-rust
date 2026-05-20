//! Симулятивная система анатомии для BRP/Bevy
//!
//! Вдохновлена Dwarf Fortress и CDDA: ткани, органы, кровотечения, инфекции.

mod components;
mod config;
pub(crate) mod systems;

// ========== Публичные типы ==========
pub use types::events::*;
pub use types::injury::{Injury, WoundSeverity, WoundType};
pub use types::location::{HitLocationRoll, HitLocationType};
pub use types::organ::{Organ, OrganCondition, OrganFunction, OrganType};
pub use types::substance::{
    Infection, InfectionSymptom, PathogenId, SubstancePool, ToxinLevel, ToxinType,
};
pub use types::tissue::{TissueLayer, TissueType};
pub use types::wound::{DamageResult, PenetrationProfile, Wound};

// ========== Компоненты ==========
pub use components::anatomy::Anatomy;
pub use components::body_part::BodyPart;
pub use components::vitals::{CharacterState, SpinalLevel, VitalStats};

// ========== Системы (для регистрации в App) ==========
pub use systems::damage::apply_damage_system;
pub use systems::healing::healing_tick_system;
pub use systems::vitals::update_vitals_system;

// ========== Конфигурация ==========
pub use config::constants::*;

mod types;

// ========== Фабричные функции ==========

/// Быстрая проверка: жив ли персонаж
#[inline]
pub fn is_character_alive(anatomy: &Anatomy) -> bool {
    anatomy.is_alive()
}

// /// Расчёт урона с полной симуляцией (выносится в систему, но полезно для тестов)
// #[cfg(feature = "debug_tools")]
// pub fn simulate_damage(
//     anatomy: &mut Anatomy,
//     location: HitLocationType,
//     damage: i32,
//     wound_type: WoundType,
//     penetration_mm: f32,
// ) -> DamageResult {
//     anatomy.apply_damage_detailed(location, damage, wound_type, penetration_mm)
// }
