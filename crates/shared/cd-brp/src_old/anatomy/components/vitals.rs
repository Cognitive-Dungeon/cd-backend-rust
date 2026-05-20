use bevy::{ecs::component::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::anatomy::{DF_PAIN_UNCONSCIOUS_THRESHOLD, SHOCK_CONSCIOUSNESS_MULTIPLIER};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpinalLevel {
    Cervical,
    Thoracic,
    Lumbar,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum CharacterState {
    Healthy,
    Wounded,
    Unconscious,
    Dead,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
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

impl VitalStats {
    /// Быстрый переход в состояние клинической смерти
    pub fn transition_to_dead(&mut self) {
        self.consciousness = 0.0;
        self.state = CharacterState::Dead;
    }

    /// Пересчитывает сознание и состояние на основе агрегированной боли и пула веществ
    pub fn recalculate_state(
        &mut self,
        total_pain: f32,
        substances: &crate::anatomy::SubstancePool,
    ) {
        self.pain = total_pain;

        // 1. Уровень гиповолемического шока
        let blood_loss_percent = substances.blood_loss_percent();
        self.shock_level = blood_loss_percent;

        // 2. Расчет пенальти к сознанию
        let pain_penalty = (self.pain / DF_PAIN_UNCONSCIOUS_THRESHOLD).clamp(0.0, 1.0);
        let shock_penalty = (blood_loss_percent * SHOCK_CONSCIOUSNESS_MULTIPLIER).clamp(0.0, 1.0);

        // Сознание падает от наихудшего фактора (или боль, или кровопотеря)
        self.consciousness = 1.0 - pain_penalty.max(shock_penalty);

        // 3. Обновление геймплейного статуса
        if self.consciousness <= 0.0 {
            self.state = CharacterState::Unconscious;
        } else if self.pain > 0.0 || substances.blood_loss_rate > 0.0 {
            self.state = CharacterState::Wounded;
        } else {
            self.state = CharacterState::Healthy;
        }
    }
}
