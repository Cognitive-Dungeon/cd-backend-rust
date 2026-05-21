//! Модуль определения локаций попаданий (Hit Locations, стр. 52-54).

use crate::types::{BodyPlan, HitLocation};

/// Чистая функция определения локации по броску D20 (стр. 53).
/// Требует бросок d20 (от 1 до 20).
/// Возвращает None, если для данного плана тела локации не используются (например, Formless).
#[must_use]
pub const fn determine_hit_location(body_plan: BodyPlan, d20_roll: u8) -> Option<HitLocation> {
    use HitLocation::*;

    // Защита от некорректных бросков (на сервере RNG должен выдавать 1-20)
    let roll = if d20_roll == 0 {
        1
    } else if d20_roll > 20 {
        20
    } else {
        d20_roll
    };

    match body_plan {
        // Гуманоид (Humanoid) - Стандартная таблица BRP (Стр. 53)
        BodyPlan::Humanoid => Some(match roll {
            1..=4 => RightLeg,
            5..=8 => LeftLeg,
            9..=11 => Abdomen,
            12..=15 => Chest,
            16..=17 => RightArm,
            18 => LeftArm,
            19..=20 => Head,
            _ => unreachable!(),
        }),

        // Четвероногие (Four-Legged) - Животные вроде лошадей, собак
        BodyPlan::FourLegged => Some(match roll {
            1..=2 => RightHindleg,
            3..=4 => LeftHindleg,
            5..=7 => Hindquarters,
            8..=10 => Forequarters,
            11..=13 => RightForeleg,
            14..=16 => LeftForeleg,
            17..=20 => Head,
            _ => unreachable!(),
        }),

        // Крылатые гуманоиды (Winged Humanoid)
        BodyPlan::WingedHumanoid => Some(match roll {
            1..=3 => RightLeg,
            4..=6 => LeftLeg,
            7..=9 => Abdomen,
            10 => Chest,
            11..=12 => RightWing,
            13..=14 => LeftWing,
            15..=16 => RightArm,
            17..=18 => LeftArm,
            19..=20 => Head,
            _ => unreachable!(),
        }),

        // Бесформенные (Formless) - Слизь, элементали. У них нет уязвимых зон.
        BodyPlan::Formless => None,

        // Для остальных планов тела (Snake, MultiLimbed и т.д.)
        // В MMO часто используют fallback на Body, если детальная таблица не задана,
        // либо расписывают полную матрицу из бестиария.
        _ => Some(Body),
    }
}

/// Модификатор HP для конкретной локации (стр. 53).
/// В BRP каждая локация имеет свои HP, зависящие от общего Max HP.
#[must_use]
pub const fn location_hp_fraction(location: HitLocation, total_hp: u16) -> u16 {
    // В Rust мы не используем float для доменной логики, применяем целочисленные дроби BRP
    use HitLocation::*;

    // Формулы из рулбука (базируются на долях от Total HP)
    match location {
        // Каждая нога, рука, живот, голова = 1/3 от Max HP (округление вверх)
        RightLeg | LeftLeg | RightArm | LeftArm | Abdomen | Head | RightHindleg | LeftHindleg
        | RightForeleg | LeftForeleg | RightWing | LeftWing => {
            (total_hp + 2) / 3 // Целочисленное деление с округлением вверх (ceil)
        }

        // Грудь (Chest) и туловища животных = 4/10 (или 40%) от Max HP (округление вверх)
        Chest | Hindquarters | Forequarters | Body => (total_hp * 4 + 9) / 10,

        // Хвост = 1/4 от Max HP
        Tail => (total_hp + 3) / 4,
    }
}
