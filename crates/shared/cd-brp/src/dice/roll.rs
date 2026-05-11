use crate::dice::DamageModifier;
use rand::{Rng, RngExt};

/// Бросает модификатор урона и возвращает результат со знаком
pub fn roll_modifier<R: Rng + ?Sized>(modifier: DamageModifier, rng: &mut R) -> i32 {
    if modifier.is_none() {
        return 0;
    }

    let sign = match modifier.sign {
        crate::dice::Sign::Positive => 1,
        crate::dice::Sign::Negative => -1,
        crate::dice::Sign::None => return 0,
    };

    let faces = modifier.dice.faces() as i32;
    let mut total = 0i32;

    for _ in 0..modifier.count {
        total += rng.random_range(1..=faces);
    }

    total * sign
}

/// Бросок базового урона с применением модификатора
pub fn roll_damage<R: Rng + ?Sized>(
    base_damage: i32,
    modifier: DamageModifier,
    rng: &mut R,
) -> i32 {
    base_damage + roll_modifier(modifier, rng)
}

/// Утилита для тестов: детерминированный бросок с сидом
#[cfg(test)]
pub fn roll_with_seed(modifier: DamageModifier, seed: u64) -> i32 {
    use rand::SeedableRng;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    roll_modifier(modifier, &mut rng)
}
