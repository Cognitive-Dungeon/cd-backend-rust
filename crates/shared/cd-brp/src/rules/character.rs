// src/rules/character.rs
use crate::domain::character::DerivedStats;
use crate::domain::chars::CharacteristicBlock;
use crate::progression::ExperienceBonus;
use crate::types::{
    DamageModifier, DieType, Edu, GameSessionConfig, HitPoints, HpCalculationRule, Int,
    ModifierSign, PowerLevel, PowerPoints, Stat,
};

/// Правило: Вычисление производных статов (Стр. 22)
pub fn calculate_derived_stats(
    stats: &CharacteristicBlock,
    config: &GameSessionConfig,
) -> DerivedStats {
    let sum_con_siz = stats.con.get() as f32 + stats.siz.get() as f32;

    let hp_val = match config.hp_calculation {
        HpCalculationRule::Average => (sum_con_siz / 2.0).ceil() as i16,
        HpCalculationRule::Total => sum_con_siz as i16,
    };

    let str_siz = stats.str.get() + stats.siz.get();
    let dmg_mod = match str_siz {
        0..=12 => DamageModifier::Modifier {
            sign: ModifierSign::Negative,
            count: 1,
            dice: DieType::D6,
        },
        13..=16 => DamageModifier::Modifier {
            sign: ModifierSign::Negative,
            count: 1,
            dice: DieType::D4,
        },
        17..=24 => DamageModifier::None,
        25..=32 => DamageModifier::Modifier {
            sign: ModifierSign::Positive,
            count: 1,
            dice: DieType::D4,
        },
        33..=40 => DamageModifier::Modifier {
            sign: ModifierSign::Positive,
            count: 1,
            dice: DieType::D6,
        },
        _ => {
            let extra = str_siz.saturating_sub(40);
            let dice_count = 1 + (extra as f32 / 16.0).ceil() as u8;
            DamageModifier::Modifier {
                sign: ModifierSign::Positive,
                count: dice_count,
                dice: DieType::D6,
            }
        }
    };

    DerivedStats {
        max_hp: HitPoints::new(hp_val),
        max_mp: PowerPoints::new(stats.pow.get() as i16),
        damage_modifier: dmg_mod,
        base_movement: crate::types::MovementRate(10),
        experience_bonus: ExperienceBonus::new((stats.int.get() as f32 / 2.0).ceil() as u16),
        major_wound_threshold: HitPoints::new((hp_val as f32 / 2.0).ceil() as i16),
    }
}

/// Правило: Бюджет профессиональных очков (Стр. 25)
pub fn calculate_professional_budget(power_level: PowerLevel, edu: Option<Stat<Edu>>) -> u16 {
    if let Some(edu_stat) = edu {
        let multiplier = match power_level {
            PowerLevel::Normal => 20,
            PowerLevel::Heroic => 25,
            PowerLevel::Epic => 30,
            PowerLevel::Superhuman => 40,
        };
        edu_stat.get() * multiplier
    } else {
        match power_level {
            PowerLevel::Normal => 250,
            PowerLevel::Heroic => 325,
            PowerLevel::Epic => 400,
            PowerLevel::Superhuman => 500,
        }
    }
}

/// Правило: Бюджет личных очков (Стр. 25)
pub fn calculate_personal_budget(
    int: Stat<Int>,
    power_level: PowerLevel,
    use_increased: bool,
) -> u16 {
    let multiplier = if use_increased {
        match power_level {
            PowerLevel::Normal => 10,
            PowerLevel::Heroic => 15,
            PowerLevel::Epic => 20,
            PowerLevel::Superhuman => 25,
        }
    } else {
        10
    };
    int.get() * multiplier
}
