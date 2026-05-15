pub mod creature;
pub mod furniture;
pub mod material;
pub mod spell;

pub use creature::{CreatureDef, CreatureId};
pub use furniture::{FurnitureDef, FurnitureId};
pub use material::MaterialDef;
pub use spell::{DamageType, SpellDef, SpellEffect, SpellId, SpellTarget};

#[derive(Debug, thiserror::Error)]
#[error("invalid numeric ID: {0}")]
pub struct ParseIdError(pub String);

/// Макрос для автоматической генерации строгих ID (Newtype pattern).
/// Реализует FromStr и Display, убирая boilerplate.
macro_rules! define_id {
    ($name:ident) => {
        use bevy::reflect::Reflect;
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Reflect)]
        #[serde(transparent)]
        pub struct $name(pub u32);

        impl std::str::FromStr for $name {
            type Err = $crate::defs::ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse::<u32>()
                    .map(Self)
                    .map_err(|_| $crate::defs::ParseIdError(s.to_string()))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// Делаем макрос доступным внутри подмодулей (creature, furniture и т.д.)
pub(crate) use define_id;
