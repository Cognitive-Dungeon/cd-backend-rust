use serde::{Deserialize, Serialize};

use crate::anatomy::{TissueType, WoundSeverity, WoundType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenetrationProfile {
    pub depth_mm: f32,
    pub tip_type: WoundType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wound {
    pub wound_type: WoundType,
    pub severity: WoundSeverity,
    pub affected_tissues: Vec<TissueType>,
    pub depth: f32,
    pub bleeding_rate: f32,
    pub pain_level: f32,
    pub infection_risk: f32,
    pub created_at: f64,
}

#[derive(Debug, Clone)]
pub enum DamageResult {
    Missed,
    Blocked,
    Hit {
        damage_dealt: i32,
        bleeding_added: f32,
        pain_caused: f32,
    },
}

impl DamageResult {
    pub fn damage_dealt(&self) -> i32 {
        match self {
            Self::Hit { damage_dealt, .. } => *damage_dealt,
            _ => 0,
        }
    }
}
