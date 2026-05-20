use serde::{Deserialize, Serialize};

/// Категории отличительных черт внешности (для генерации)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureCategory {
    HairOnHead,
    FacialHair,
    FacialFeature,
    Expression,
    Clothes,
    Bearing,
    Speech,
    ArmsAndHands,
    Torso,
    LegsAndFeet,
}

/// Базовые типы личности (дают +20% к определенному набору навыков)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PersonalityType {
    Brutal,
    Skilled,
    Cunning,
    Charming,
    Custom { name: String },
}
