use serde::{Deserialize, Serialize};

/// Кто управляет данной сущностью
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterControl {
    Player,
    Gamemaster,
}

/// Ведущая рука персонажа (важно для штрафов при бое парным оружием / травмах)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Handedness {
    #[default]
    Right,
    Left,
    Ambidextrous,
}

/// Ведущая нога персонажа (важно для расчета травм и механик движения/ударов)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Footedness {
    #[default]
    Right,
    Left,
    Ambidextrous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocialRank {
    Slave,
    Tribesperson,
    LowerClass,
    LowerMiddleClass,
    MiddleClass,
    UpperMiddleClass,
    UpperClass,
    Nobility,
    Monarchy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPlan {
    Humanoid,
    Formless,
    FourLegged,
    FourLeggedHumanoid,
    FourLeggedWithTail,
    GiantFourLeggedWithTail,
    MultiLimbed,
    Snake,
    TwoLeggedWithTail,
    Winged,
    WingedFourLegged,
    WingedFourLeggedWithTail,
    WingedHumanoid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitLocation {
    // Humanoid
    Head,
    Chest,
    Abdomen,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
    // Animals & Monsters
    Hindquarters,
    Forequarters,
    RightHindleg,
    LeftHindleg,
    RightForeleg,
    LeftForeleg,
    Tail,
    RightWing,
    LeftWing,
    Body,
}

impl BodyPlan {
    /// Возвращает статический список доступных локаций для конкретного плана тела.
    /// Гарантирует, что система боя не сможет нанести урон в "Хвост" Гуманоиду.
    pub const fn available_locations(self) -> &'static [HitLocation] {
        use HitLocation::*;
        match self {
            BodyPlan::Humanoid => &[RightLeg, LeftLeg, Abdomen, Chest, RightArm, LeftArm, Head],
            BodyPlan::WingedHumanoid => &[
                RightLeg, LeftLeg, Abdomen, Chest, RightArm, LeftArm, Head, RightWing, LeftWing,
            ],
            BodyPlan::Formless => &[Body],
            BodyPlan::FourLegged => &[
                RightHindleg,
                LeftHindleg,
                Hindquarters,
                Forequarters,
                RightForeleg,
                LeftForeleg,
                Head,
            ],
            BodyPlan::FourLeggedHumanoid => &[
                RightHindleg,
                LeftHindleg,
                Hindquarters,
                Forequarters,
                RightArm,
                LeftArm,
                Head,
                Chest,
            ],
            BodyPlan::FourLeggedWithTail | BodyPlan::GiantFourLeggedWithTail => &[
                Tail,
                RightHindleg,
                LeftHindleg,
                Hindquarters,
                Forequarters,
                RightForeleg,
                LeftForeleg,
                Head,
            ],
            BodyPlan::MultiLimbed => &[RightArm, LeftArm, Body, Head], // Упрощено для примера (щупальца)
            BodyPlan::Snake => &[Tail, Body, Head],
            BodyPlan::TwoLeggedWithTail => &[
                Tail, RightLeg, LeftLeg, Abdomen, Chest, RightArm, LeftArm, Head,
            ],
            BodyPlan::Winged => &[RightLeg, LeftLeg, Body, RightWing, LeftWing, Head],
            BodyPlan::WingedFourLegged | BodyPlan::WingedFourLeggedWithTail => &[
                Tail,
                RightHindleg,
                LeftHindleg,
                Hindquarters,
                Forequarters,
                RightWing,
                LeftWing,
                RightForeleg,
                LeftForeleg,
                Head,
            ],
        }
    }
}
