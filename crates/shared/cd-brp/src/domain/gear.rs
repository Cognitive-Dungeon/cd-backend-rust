// domain/gear.rs

use serde::{Deserialize, Serialize};

use crate::{DefId, DieType, HandednessReq, HitPoints, SpecialSuccessEffect, WeaponClass};

/// Статический чертеж (Blueprint) оружия.
/// Загружается в память сервера один раз при старте.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponBlueprint {
    pub id: DefId, // Строгий хэш-ID
    pub class: WeaponClass,
    pub handedness: HandednessReq,
    pub base_damage_dice: (u8, DieType),
    pub flat_damage_bonus: i16,
    pub special_effect: SpecialSuccessEffect,
    pub base_hp: HitPoints,
    pub can_parry: bool,
    pub min_str: u8,
    pub min_dex: u8,
}
