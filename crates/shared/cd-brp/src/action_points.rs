use crate::anatomy::{Anatomy, HitLocationType, Injury};
use crate::characteristics::Characteristics;
use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ActionPoints {
    pub current: i32,
}

impl ActionPoints {
    /// Рассчитывает максимум AP на основе характеристик и текущих травм (Anatomy)
    pub fn calculate_max(chars: &Characteristics, anatomy: &Anatomy) -> i32 {
        // База: DEX / 2 (округляем вверх) + бонус за скорость
        let mut base_ap = (chars.dex as f32 / 2.0).ceil() as i32;

        // Если ноги сломаны — AP режется пополам (штраф из Witcher/TRP)
        let left_leg = &anatomy.parts[HitLocationType::LeftLeg];
        let right_leg = &anatomy.parts[HitLocationType::RightLeg];

        let is_leg_fractured = left_leg.injuries.contains(&Injury::Fractured)
            || right_leg.injuries.contains(&Injury::Fractured);

        let is_leg_severed = left_leg.is_destroyed() || right_leg.is_destroyed();

        if is_leg_severed {
            base_ap /= 4; // Ползком
        } else if is_leg_fractured {
            base_ap /= 2; // Хромает
        }

        // Защита от 0
        base_ap.max(1)
    }

    /// Удобный метод для списания AP в бою
    pub fn try_consume(&mut self, amount: i32) -> Result<(), &'static str> {
        if self.current >= amount {
            self.current -= amount;
            Ok(())
        } else {
            Err("Not enough Action Points")
        }
    }
}
