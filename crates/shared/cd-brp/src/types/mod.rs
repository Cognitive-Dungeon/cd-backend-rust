//! Строгие доменные типы для BRP.

pub mod character;
pub mod combat;
pub mod config;
pub mod core;
pub mod dice;
pub mod environment;
pub mod equipment;
pub mod error;
pub mod features;
pub mod markers;
pub mod pools;
pub mod powers;
pub mod profession;
pub mod psychology;
pub mod skills;
pub mod stats;
pub mod vehicles;

// --- Реэкспорты для удобного доступа через `cd_brp::...` ---
pub use character::*;
pub use combat::*;
pub use config::*;
pub use core::*;
pub use dice::{D100Roll, DamageModifier, DiceExpression, DieType, ModifierSign};
pub use environment::*;
pub use equipment::*;
pub use error::TypeError;
pub use features::*;
pub use markers::*;
pub use pools::*;
pub use powers::*;
pub use profession::*;
pub use psychology::*;
pub use skills::*;
pub use stats::*;
pub use vehicles::*;
