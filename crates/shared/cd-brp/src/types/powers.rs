use crate::types::core::PowerLevel;
use serde::{Deserialize, Serialize};

/// Обертка над PowerLevel для модуля Сил с расчетом "веса бюджета"
impl PowerLevel {
    /// Возвращает "вес" уровня для композиции множественных наборов сил
    /// (MD: Multiple Power Sets budget)
    pub const fn power_weight(self) -> u8 {
        match self {
            Self::Normal => 1,
            Self::Heroic => 2,
            Self::Epic => 3,
            Self::Superhuman => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MagicSpell {
    Blast,
    Change,
    ConjureElemental,
    Control,
    Countermagic,
    Dark,
    Diminish,
    Dispel,
    Dull,
    Enhance,
    Fire,
    Frost,
    Heal,
    Illusion,
    Invisibility,
    Lift,
    Light,
    Lightning,
    Perception,
    Protection,
    Resistance,
    Seal,
    Sharpen,
    SpeakToMind,
    Teleport,
    Unseal,
    Vision,
    Wall,
    Ward,
    Wounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mutation {
    Adaptability,
    Allergy,
    BipedQuadruped,
    Camouflage,
    Colouration,
    CongenitalDisease,
    DecreasedCharacteristic,
    DiseaseCarrier,
    GroupIntelligence,
    Hands,
    Hardy,
    Hybrid,
    Imitation,
    IncreasedCharacteristic,
    KeenSense,
    Luminescence,
    MetabolicImprovement,
    MetabolicWeakness,
    NaturalArmour,
    NaturalWeaponry,
    PainSensitivity,
    Pheromone,
    ReducedSense,
    Regeneration,
    Sensitivity,
    SpeechMimicry,
    StructuralImprovement,
    StructuralWeakness,
    Venom,
    Wings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PsychicAbility {
    AstralProjection,
    AuraDetection,
    Clairvoyance,
    Cryokinesis,
    DangerSense,
    DeadCalm,
    Divination,
    EideticMemory,
    EmotionControl,
    Empathy,
    Intuition,
    Levitation,
    MindBlast,
    MindControl,
    MindShield,
    Precognition,
    Psychometry,
    Pyrokinesis,
    Sensitivity,
    Telekinesis,
    Telepathy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SorcerySpell {
    CloakOfNight,
    Leap,
    Sureness,
    BolsterTheSoul,
    InhumanPlasticity,
    LightningSpeed,
    RelentlessVitality,
    SupplenessOfTheSerpent,
    TitansStrength,
    UnearthlyBeauty,
    WisdomOfTheSage,
    HammerOfTheGods,
    HellsRazor,
    SorcerysSharpFlame,
    SorcerousArmour,
    TalonsOfTheBeast,
    UnbreakableBulwark,
    MakeFast,
    MakeWhole,
    Midnight,
    Moonrise,
    BountyOfTheSea,
    FiresOfTheSun,
    GiftOfTheEarth,
    WingsOfTheSky,
    CurseOfSorcery,
    Fury,
    InescapableBonds,
    LikenShape,
    Muddle,
    Pox,
    BrazierOfPower,
    ChainOfBeing,
    UndoSorcery,
    Ward,
    SummonDemon,
    SummonElemental,
    BirdsVision,
    BreathOfLife,
    Farsight,
    Heal,
    KeenEar,
    Refutation,
    VerminsVision,
    WitchSight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Superpower {
    Absorption,
    Adaptation,
    AlternateForm,
    Armour,
    Barrier,
    Defence,
    DensityControl,
    DiminishEnhanceCharacteristic,
    Drain,
    EnergyControl,
    EnergyProjection,
    ExtraEnergy,
    ExtraHitPoints,
    Flight,
    ForceField,
    Intangibility,
    Invisibility,
    Leap,
    Protection,
    Regeneration,
    Resistance,
    Sidekick,
    SizeChange,
    SnareProjection,
    Stretching,
    SuperCharacteristic,
    SuperMovement,
    SuperSense,
    SuperSkill,
    SuperSpeed,
    Teleport,
    Transfer,
    UnarmedCombat,
    WeatherControl,
}

/// Агрегирующий Enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "power_type", content = "power")]
pub enum PowerType {
    Magic(MagicSpell),
    Mutation(Mutation),
    Psychic(PsychicAbility),
    Sorcery(SorcerySpell),
    Superpower(Superpower),
}
