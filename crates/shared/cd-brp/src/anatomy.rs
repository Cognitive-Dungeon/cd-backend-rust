use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HitLocationType {
    RightLeg,
    LeftLeg,
    Abdomen,
    Chest,
    RightArm,
    LeftArm,
    Head,
}

impl HitLocationType {
    /// Бросок D20 для выбора зоны попадания по гуманоиду
    pub fn roll_humanoid(d20_roll: i32) -> Self {
        match d20_roll {
            1..=4 => Self::RightLeg,
            5..=8 => Self::LeftLeg,
            9..=11 => Self::Abdomen,
            12 => Self::Chest,
            13..=15 => Self::RightArm,
            16..=18 => Self::LeftArm,
            _ => Self::Head, // 19-20
        }
    }

    /// Доля от общего пула ХП для этой части тела (правила BRP, стр 14)
    pub fn hp_fraction(&self) -> f32 {
        match self {
            Self::RightLeg | Self::LeftLeg | Self::Abdomen | Self::Head => 1.0 / 3.0,
            Self::Chest => 4.0 / 10.0,
            Self::RightArm | Self::LeftArm => 1.0 / 4.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Injury {
    Bleeding,
    Fractured, // Сломано (режет AP)
    Severed,   // Оторвано/Уничтожено
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    pub location: HitLocationType,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub injuries: Vec<Injury>,
}

impl BodyPart {
    pub fn new(total_hp: i32, location: HitLocationType, armor: i32) -> Self {
        let max_hp = (total_hp as f32 * location.hp_fraction()).ceil() as i32;
        Self {
            location,
            hp: max_hp,
            max_hp,
            armor,
            injuries: Vec::new(),
        }
    }

    /// Конечность выведена из строя (упал, выронил оружие)
    pub fn is_useless(&self) -> bool {
        self.hp <= 0 || self.injuries.contains(&Injury::Severed)
    }

    /// Конечность уничтожена безвозвратно (урон х2 от максимума)
    pub fn is_destroyed(&self) -> bool {
        self.hp <= -self.max_hp || self.injuries.contains(&Injury::Severed)
    }
}

/// ECS-компонент, заменяющий примитивный Stats
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Anatomy {
    pub total_hp: i32,
    pub current_hp: i32,
    pub parts: HashMap<HitLocationType, BodyPart>,
}

impl Anatomy {
    /// Создает человеческую анатомию на основе базовых ХП (CON + SIZ / 2)
    pub fn new_humanoid(total_hp: i32) -> Self {
        let mut parts = HashMap::new();
        for loc in [
            HitLocationType::RightLeg,
            HitLocationType::LeftLeg,
            HitLocationType::Abdomen,
            HitLocationType::Chest,
            HitLocationType::RightArm,
            HitLocationType::LeftArm,
            HitLocationType::Head,
        ] {
            parts.insert(loc, BodyPart::new(total_hp, loc, 0));
        }

        Self {
            total_hp,
            current_hp: total_hp,
            parts,
        }
    }

    /// Смерть наступает при <= 0 общих ХП, либо при уничтожении головы/груди/живота
    pub fn is_alive(&self) -> bool {
        if self.current_hp <= 0 {
            return false;
        }

        for critical_loc in [
            HitLocationType::Head,
            HitLocationType::Chest,
            HitLocationType::Abdomen,
        ] {
            if let Some(part) = self.parts.get(&critical_loc)
                && part.is_destroyed()
            {
                return false;
            }
        }
        true
    }

    /// Нанесение урона в конкретную часть тела
    pub fn apply_damage(&mut self, location: HitLocationType, raw_damage: i32) -> i32 {
        if let Some(part) = self.parts.get_mut(&location) {
            let actual_damage = (raw_damage - part.armor).max(0);

            // В BRP конечность не может получить больше 2x урона от своего максимума за один удар
            let max_possible_damage = part.max_hp * 2;
            let final_damage = actual_damage.min(max_possible_damage);

            part.hp -= final_damage;
            self.current_hp -= final_damage;

            if part.is_destroyed() && !part.injuries.contains(&Injury::Severed) {
                part.injuries.push(Injury::Severed);
            } else if part.is_useless() && !part.injuries.contains(&Injury::Fractured) {
                part.injuries.push(Injury::Fractured);
            }

            return final_damage;
        }
        0
    }
}
