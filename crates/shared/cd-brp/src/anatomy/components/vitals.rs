use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpinalLevel {
    Cervical,
    Thoracic,
    Lumbar,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterState {
    Healthy,
    Wounded,
    Unconscious,
    Dead,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct VitalStats {
    pub pain: f32,
    pub shock_level: f32,
    pub consciousness: f32,
    pub state: CharacterState,
}

impl Default for VitalStats {
    fn default() -> Self {
        Self {
            pain: 0.0,
            shock_level: 0.0,
            consciousness: 1.0,
            state: CharacterState::Healthy,
        }
    }
}
