//! Ядро разрешения действий: матрицы боя, модификаторы, проверка бросков.

pub mod combat_matrix;
pub mod damage;
pub mod modifiers;
pub mod resolution;

pub use combat_matrix::{ExchangeOutcome, TargetHitType};
pub use damage::{DamageApplication, calculate_actual_damage};
pub use modifiers::{
    apply_difficulty, calculate_category_bonus, calculate_effective_skill,
    calculate_simple_category_bonus,
};
pub use resolution::{BrpThresholds, resolve_resistance, resolve_skill};
