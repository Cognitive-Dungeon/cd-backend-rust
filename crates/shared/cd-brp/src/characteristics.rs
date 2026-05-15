use crate::dice::{DamageModifier, DiceType, Sign};
use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Component)]
pub struct Characteristics {
    pub str: i32, // Strength
    pub con: i32, // Constitution
    pub siz: i32, // Size
    pub int: i32, // Intelligence
    pub pow: i32, // Power
    pub dex: i32, // Dexterity
    pub cha: i32, // Charisma
    pub edu: i32, // Education
}

impl Characteristics {
    pub fn new(
        str: i32,
        con: i32,
        siz: i32,
        int: i32,
        pow: i32,
        dex: i32,
        cha: i32,
        edu: i32,
    ) -> Self {
        Self {
            str,
            con,
            siz,
            int,
            pow,
            dex,
            cha,
            edu,
        }
    }

    /// Общие хитпоинты: среднее от CON и SIZ с округлением вверх
    pub fn max_hit_points(&self) -> i32 {
        (self.con + self.siz + 1) / 2
    }

    /// Базовый модификатор урона на основе STR + SIZ
    pub fn damage_modifier(&self) -> DamageModifier {
        let total = self.str + self.siz;
        use {DiceType::*, Sign::*};

        match total {
            2..=12 => DamageModifier::new(Negative, 1, D6),
            13..=16 => DamageModifier::new(Negative, 1, D4),
            17..=24 => DamageModifier::NONE,
            25..=32 => DamageModifier::new(Positive, 1, D4),
            33..=40 => DamageModifier::new(Positive, 1, D6),
            41..=56 => DamageModifier::new(Positive, 2, D6),
            57..=72 => DamageModifier::new(Positive, 3, D6),
            _ => DamageModifier::new(Positive, 4, D6), // монстры / эпик
        }
    }

    /// Бонус к опыту: половина INT с округлением вверх
    pub fn experience_bonus(&self) -> i32 {
        (self.int + 1) / 2
    }

    /// Валидация диапазона характеристик (опционально)
    pub fn validate(&self) -> Result<(), crate::BrpError> {
        use crate::BrpError::CharacteristicOutOfRange;
        for (_name, value) in [
            ("STR", self.str),
            ("CON", self.con),
            ("SIZ", self.siz),
            ("INT", self.int),
            ("POW", self.pow),
            ("DEX", self.dex),
            ("CHA", self.cha),
            ("EDU", self.edu),
        ] {
            if !(1..=100).contains(&value) {
                return Err(CharacteristicOutOfRange { value });
            }
        }
        Ok(())
    }
}

impl Default for Characteristics {
    fn default() -> Self {
        Self::new(10, 10, 10, 10, 10, 10, 10, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_points_rounding() {
        let chars = Characteristics::new(10, 13, 14, 10, 10, 10, 10, 10);
        assert_eq!(chars.max_hit_points(), 14); // (13+14+1)/2 = 14

        let chars = Characteristics::new(10, 12, 13, 10, 10, 10, 10, 10);
        assert_eq!(chars.max_hit_points(), 13); // (12+13+1)/2 = 13
    }

    #[test]
    fn test_damage_modifier_ranges() {
        let test_cases = [
            (10, 2, "-1D6"),  // 12
            (10, 5, "-1D4"),  // 15
            (10, 10, "0"),    // 20
            (15, 12, "+1D4"), // 27
            (20, 15, "+1D6"), // 35
            (30, 20, "+2D6"), // 50
            (40, 20, "+3D6"), // 60
            (50, 30, "+4D6"), // 80
        ];

        for (str, siz, expected) in test_cases {
            let chars = Characteristics::new(str, 10, siz, 10, 10, 10, 10, 10);
            assert_eq!(chars.damage_modifier().as_str(), expected);
        }
    }

    #[test]
    fn test_experience_bonus() {
        let chars = Characteristics::new(10, 10, 10, 15, 10, 10, 10, 10);
        assert_eq!(chars.experience_bonus(), 8); // (15+1)/2

        let chars = Characteristics::new(10, 10, 10, 16, 10, 10, 10, 10);
        assert_eq!(chars.experience_bonus(), 8); // (16+1)/2 = 8 (целочисленное)
    }
}
