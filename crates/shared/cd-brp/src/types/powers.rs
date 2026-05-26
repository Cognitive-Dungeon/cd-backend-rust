use crate::{
    Characteristic, HitPoints, Meters, PowerPoints, SuccessLevel, types::core::PowerLevel,
};
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

/// Контекст и результат попытки использовать силу (каст заклинания, активация псионики).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerActivationResult {
    /// Сила успешно активирована. Возвращает уровень успеха (для эффектов) и потраченные MP.
    Success {
        level: SuccessLevel,
        mp_spent: PowerPoints,
    },
    /// Провал броска. Сила не сработала. По правилам BRP, при провале каста
    /// обычно тратится 1 MP (или половина маны, зависит от опций). Мы фиксируем потерю 1 MP.
    Failure { mp_spent: PowerPoints },
    /// Критический провал (Fumble). Может вызвать откат (Backfire), потерю всех вложенных MP
    /// или даже урон самому заклинателю.
    Fumble {
        mp_spent: PowerPoints,
        backfire_damage: Option<HitPoints>,
    },
    /// У персонажа не хватило Power Points для активации.
    NotEnoughPowerPoints,
}

/// Дальность действия способности.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerRange {
    SelfOnly,
    Touch,
    Distance(Meters),
    Sight, // В пределах видимости
}

/// Длительность действия способности.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerDuration {
    Instantaneous,     // Мгновенно (урон нанесен и всё)
    CombatRounds(u16), // N раундов
    Minutes(u16),
    Hours(u16),
    Active, // Пока кастер поддерживает концентрацию / тратит ману
}

/// Как расходуются очки магии на эту способность.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PowerCost {
    /// Фиксированная цена (например, всегда 2 MP)
    Fixed(PowerPoints),
    /// Цена за каждый уровень силы (как в заклинании Fire: 3 MP за уровень)
    PerLevel(PowerPoints),
    /// Свободная трата (от min до max), часто в псионике
    Variable { min: PowerPoints, max: PowerPoints },
}

/// Возможные защиты от способности
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerDefense {
    None,         // Нельзя избежать (кроме сопротивления магии)
    DodgeAllowed, // Можно увернуться (как от Fire)
    ParryAllowed,
    ResistanceTable(Characteristic, Characteristic), // Например, POW vs POW
}
