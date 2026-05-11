use serde::{Deserialize, Serialize};

/// Градации успеха в BRP
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SuccessLevel {
    Fumble,
    Failure,
    Success,
    Special,
    Critical,
}

impl SuccessLevel {
    /// Вычисляет градацию успеха для броска D100 (1..=100) против рейтинга навыка
    pub fn evaluate(roll: i32, skill_rating: i32) -> Self {
        // Правило BRP: 100 — это всегда Fumble (провал), 1 — всегда Critical (успех)
        if roll == 100 {
            return Self::Fumble;
        }
        if roll == 1 {
            return Self::Critical;
        }

        if roll > skill_rating {
            // Расчет порога Fumble (худшие 5% от шанса провала)
            let failure_chance = (100 - skill_rating).max(0);
            let fumble_range = (failure_chance as f32 / 20.0).ceil() as i32;
            let fumble_threshold = 101 - fumble_range.max(1); // минимум 100

            if roll >= fumble_threshold {
                return Self::Fumble;
            }
            return Self::Failure;
        }

        // Успех (roll <= skill_rating)
        let critical_threshold = (skill_rating as f32 / 20.0).ceil() as i32;
        if roll <= critical_threshold {
            return Self::Critical;
        }

        let special_threshold = (skill_rating as f32 / 5.0).ceil() as i32;
        if roll <= special_threshold {
            return Self::Special;
        }

        Self::Success
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::Special | Self::Critical)
    }
}

/// Вычисляет шанс успеха по Таблице Сопротивления (Resistance Table)
/// Формула: 50% + (Active * 5) - (Passive * 5)
pub fn resistance_chance(active: i32, passive: i32) -> i32 {
    let chance = 50 + (active * 5) - (passive * 5);
    // Шанс всегда в пределах от 1 до 99 (01 всегда успех, 00 всегда провал)
    chance.clamp(1, 99)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_levels() {
        let skill = 50;
        assert_eq!(SuccessLevel::evaluate(1, skill), SuccessLevel::Critical); // 1-3
        assert_eq!(SuccessLevel::evaluate(3, skill), SuccessLevel::Critical);
        assert_eq!(SuccessLevel::evaluate(10, skill), SuccessLevel::Special); // 4-10
        assert_eq!(SuccessLevel::evaluate(50, skill), SuccessLevel::Success); // 11-50
        assert_eq!(SuccessLevel::evaluate(97, skill), SuccessLevel::Failure); // 51-97
        assert_eq!(SuccessLevel::evaluate(98, skill), SuccessLevel::Fumble); // 98-100 (50 провал / 20 = 2.5 -> 3)
    }

    #[test]
    fn test_resistance_table() {
        assert_eq!(resistance_chance(10, 10), 50); // Равные силы = 50%
        assert_eq!(resistance_chance(15, 10), 75); // Активный сильнее на 5 = +25%
        assert_eq!(resistance_chance(10, 15), 25); // Активный слабее на 5 = -25%
    }
}
