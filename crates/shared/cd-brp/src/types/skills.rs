use crate::WeaponClass;

use super::core::DefId;
use serde::{Deserialize, Serialize};

/// Полный, строгий список навыков из рулбука (стр. 69-70).
/// Занимает минимум памяти, сравнивается за такт процессора.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    // === Фиксированные базовые навыки ===
    Appraise,
    Bargain,
    Brawl,
    Climb,
    Command,
    Demolition,
    Disguise,
    Dodge,
    FastTalk,
    FineManipulation,
    FirstAid,
    Fly,
    Gaming,
    Grapple,
    Hide,
    Insight,
    Jump,
    Listen,
    Literacy,
    MartialArts,
    Medicine,
    Navigate,
    Persuade,
    Projection,
    Psychotherapy,
    Research,
    Sense,
    SleightOfHand,
    Spot,
    Status,
    Stealth,
    Strategy,
    Swim,
    Teach,
    Throw,
    Track,

    // === Зависимые навыки (явная связь с WeaponClass) ===
    Artillery(WeaponClass),
    EnergyWeapon(WeaponClass),
    Firearm(WeaponClass),
    HeavyWeapon(WeaponClass),
    MeleeWeapon(WeaponClass),
    MissileWeapon(WeaponClass),
    Parry(WeaponClass),

    // === Навыки со специализациями (содержат ID специализации) ===
    Art(DefId),   // Напр: DefId для "Painting"
    Craft(DefId), // Напр: DefId для "Blacksmithing"
    Drive(DefId),
    HeavyMachine(DefId),
    Knowledge(DefId), // Напр: DefId для "History"
    LanguageOwn(DefId),
    LanguageOther(DefId),
    Perform(DefId),
    Pilot(DefId),
    Repair(DefId),
    Ride(DefId),
    Science(DefId),
    Shield,
    TechnicalSkill(DefId),
}
