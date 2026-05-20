//! Ядро разрешения действий: матрицы боя, модификаторы, проверка бросков.

pub mod combat_matrix;
pub mod modifiers;
pub mod resolution;

pub use combat_matrix::{ExchangeOutcome, TargetHitType};
//pub use modifiers::calculate_skill_category_bonus;
pub use resolution::{CheckResolver, ResistanceResolver};
