//! Ядро разрешения действий: матрицы боя, модификаторы, проверка бросков.

pub mod character;
pub mod combat_matrix;
pub mod damage;
pub mod encumbrance;
pub mod experience;
pub mod fatigue;
pub mod hit_locations;
pub mod modifiers;
pub mod ranged_combat;
pub mod resolution;
pub mod sanity;
pub mod skills;
pub mod strike_rank;

pub use character::{
    calculate_derived_stats, calculate_personal_budget, calculate_professional_budget,
};
pub use combat_matrix::{ExchangeOutcome, TargetHitType};
pub use damage::{DamageApplication, calculate_actual_damage};
pub use modifiers::{
    apply_difficulty, calculate_category_bonus, calculate_effective_skill,
    calculate_simple_category_bonus,
};
pub use resolution::{BrpThresholds, resolve_resistance, resolve_skill};
