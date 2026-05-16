use enum_map::Enum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Enum)]
pub enum OrganType {
    Brain,
    Heart,
    Lungs,
    Liver,
    Stomach,
    Intestines,
    Kidneys,
    Spine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Enum)]
pub enum OrganFunction {
    Vital,
    Locomotion,
    Manipulation,
    Sensory,
    Digestion,
    Detoxification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum OrganCondition {
    Bruised,
    Lacerated,
    Ruptured,
    Necrotic,
    Infected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organ {
    pub organ_type: OrganType,
    pub functions: Vec<OrganFunction>,
    pub integrity: f32,
    pub max_integrity: f32,
    pub conditions: Vec<OrganCondition>,
}
