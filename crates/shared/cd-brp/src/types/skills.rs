use crate::{VehicleCategory, WeaponClass};

use super::core::DefId;
use serde::{Deserialize, Serialize};

/// Полный, строгий список навыков из рулбука (стр. 69-70).
/// Занимает минимум памяти, сравнивается за такт процессора.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    Medicine,
    Navigate,
    Persuade,
    Projection,
    Psychotherapy,
    Research,
    Sense,
    SleightOfHand,
    Spot,
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
    // TODO: Вывести отдельное перечесление для типов щитов
    Shield(WeaponClass),

    // === Навыки со специализациями (содержат ID специализации) ===
    Art(ArtType),
    Craft(CraftType),
    Drive(VehicleCategory),
    Etiquette(DefId), // ID фракции/расы
    HeavyMachine(HeavyMachineType),
    Knowledge(KnowledgeType),
    LanguageOwn(DefId),   // ID языка
    LanguageOther(DefId), // ID языка
    Literacy(DefId),      // ID языка
    MartialArts(MartialArtsType),
    Perform(PerformType),
    Pilot(VehicleCategory),
    Repair(RepairType),
    Ride(DefId), // ID существа/категории маунта
    Science(ScienceType),
    Status(DefId), // Статус привязан к обществу/гильдии (DefId)
    TechnicalSkill(TechnicalType),
}

/// Виды искусства (Art)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtType {
    Calligraphy,
    Composing,
    ConceptualArt,
    DigitalArt,
    Drawing,
    Painting,
    Photography,
    Poetry,
    Sculpture,
    Sketching,
    Songwriting,
    Writing,
}

/// Виды ремесла (Craft)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CraftType {
    Blacksmithing,
    Carpentry,
    Ceramics,
    Cooking,
    Leatherworking,
    Locksmithing,
    Metallurgy,
    Stonemasonry,
}

/// Знания (Knowledge)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeType {
    AcademicLore,
    Accounting,
    Anthropology,
    Archaeology,
    ArtHistory,
    BlasphemousLore, // База 00%, не улучшается опытом!
    Business,
    Espionage,
    Folklore,
    History,
    Law,
    Linguistics,
    Literature,
    Occult,
    Philosophy,
    Politics,
    Region,
    Religion,
    Streetwise,
}

/// Выступления (Perform)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformType {
    Act,
    ConductOrchestra,
    Dance,
    Juggle,
    Orate,
    PlayInstrument,
    Recite,
    Ritual,
    Sing,
}

/// Тяжелые механизмы (Heavy Machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyMachineType {
    ArmoredVehicle,
    Boiler,
    Bulldozer,
    Crane,
    Engine,
    Turbine,
}

/// Ремонт (Repair)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairType {
    Electrical,
    Electronic,
    Engineering,
    Hydroelectric,
    Mechanical,
    Structural,
    Quantum,
}

/// Науки (Science)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScienceType {
    Astronomy,
    BehavioralScience,
    Biology,
    Botany,
    Chemistry,
    Cryptography,
    Genetics,
    Geology,
    Mathematics,
    Meteorology,
    NaturalHistory,
    Pharmacology,
    Physics,
    Planetology,
    Psychology,
    QuantumMechanics,
    Xenobiology,
    Zoology,
}

/// Технические навыки (Technical Skill)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalType {
    Clockwork,
    Computers,
    Cybernetics,
    Electronics,
    Robotics,
    Sensors,
    SiegeEngines,
    Traps,
}

/// Боевые искусства (Martial Arts)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MartialArtsType {
    Boxing,
    Capoeira,
    Escrima,
    Fencing,
    JeetKuneDo,
    Karate,
    Kenjutsu,
    Kickboxing,
    KungFu,
    Kyujutsu,
    Pugilism,
    Savate,
}

// Для языков, этикета и статуса в MMO обычно заводят ID фракций или рас,
// но пока можно сделать заглушки или использовать строгие типы рас.
