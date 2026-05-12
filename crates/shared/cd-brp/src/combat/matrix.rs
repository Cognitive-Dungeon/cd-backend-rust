use super::types::{AttackResolution, EffectiveHit};
use crate::rolls::SuccessLevel;

// Индексы для LUT
const fn atk_idx(level: SuccessLevel) -> usize {
    match level {
        SuccessLevel::Fumble => 0,
        SuccessLevel::Failure => 1,
        SuccessLevel::Success => 2,
        SuccessLevel::Special => 3,
        SuccessLevel::Critical => 4,
    }
}

const fn def_idx(level: Option<SuccessLevel>) -> usize {
    match level {
        None => 0,
        Some(SuccessLevel::Fumble) => 1,
        Some(SuccessLevel::Failure) => 2,
        Some(SuccessLevel::Success) => 3,
        Some(SuccessLevel::Special) => 4,
        Some(SuccessLevel::Critical) => 5,
    }
}

/// Декларативная таблица: [Атака][Защита] → (Тип попадания, Урон по предмету)
/// Строго соответствует BRP UGE стр. 147
const RESOLUTION_LUT: [[(EffectiveHit, i32); 6]; 5] = [
    // Fumble Attack
    [
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
    ],
    // Failure Attack
    [
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
        (EffectiveHit::MissOrBlocked, 0),
    ],
    // Success Attack
    [
        (EffectiveHit::Normal, 0),        // vs None
        (EffectiveHit::Normal, 0),        // vs Fumble
        (EffectiveHit::Normal, 0),        // vs Failure
        (EffectiveHit::MissOrBlocked, 0), // vs Success — полная блокировка
        (EffectiveHit::MissOrBlocked, 0), // vs Special — атака отбита, урон по щиту 0 (бьём по атакующему)
        (EffectiveHit::MissOrBlocked, 0), // vs Critical — атака отбита, урон по щиту 0
    ],
    // Special Attack
    [
        (EffectiveHit::Special, 0),       // vs None
        (EffectiveHit::Special, 0),       // vs Fumble
        (EffectiveHit::Special, 0),       // vs Failure
        (EffectiveHit::Normal, 2),        // vs Success — частичный парир, 2 урона по щиту
        (EffectiveHit::MissOrBlocked, 0), // vs Special — отбито
        (EffectiveHit::MissOrBlocked, 1), // vs Critical — отбито, урон по щиту 1
    ],
    // Critical Attack
    [
        (EffectiveHit::Critical, 0),      // vs None — крит
        (EffectiveHit::Critical, 0),      // vs Fumble — крит
        (EffectiveHit::Critical, 0),      // vs Failure — крит
        (EffectiveHit::Special, 4),       // vs Success — спец + 4 урона по щиту
        (EffectiveHit::Normal, 2),        // vs Special — норм + 2 урона по щиту
        (EffectiveHit::MissOrBlocked, 0), // vs Critical — отбито
    ],
];

pub fn resolve(atk: SuccessLevel, def: Option<SuccessLevel>) -> AttackResolution {
    let (hit, damage) = RESOLUTION_LUT[atk_idx(atk)][def_idx(def)];
    AttackResolution {
        hit,
        defense_item_damage: damage,
    }
}
