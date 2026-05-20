mod roll;
mod types;

pub use roll::{roll_damage, roll_modifier};
pub use types::{DamageModifier, DiceType, Sign};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_damage_modifier_parse() {
        assert_eq!(
            DamageModifier::parse("+1D6"),
            Some(DamageModifier::new(Sign::Positive, 1, DiceType::D6))
        );
        assert_eq!(DamageModifier::parse("invalid"), None);
        assert_eq!(DamageModifier::parse("0"), Some(DamageModifier::NONE));
    }

    #[test]
    fn test_roll_modifier_range() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let modifier = DamageModifier::new(Sign::Positive, 2, DiceType::D6);

        for _ in 0..100 {
            let result = roll_modifier(modifier, &mut rng);
            assert!((2..=12).contains(&result), "Roll {} out of range", result);
        }
    }

    #[test]
    fn test_negative_modifier() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(123);
        let modifier = DamageModifier::new(Sign::Negative, 1, DiceType::D4);
        let result = roll_modifier(modifier, &mut rng);
        assert!((-4..=-1).contains(&result));
    }
}
