use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

/// Какую таблицу использовать
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FumbleTable {
    MeleeAttack,
    MeleeParry,
    MissileAttack,
    NaturalWeapon, // Когти, укусы, кулаки
}

/// Конкретные эффекты, которые ECS-движок должен применить к сущности
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FumbleEffect {
    /// Пропустить следующий раунд
    LoseNextRound,
    /// Пропустить 1D3 раунда (Stunned / Helpless)
    LoseRounds(i32),
    /// Упасть
    FallProne,
    /// Выронить оружие   
    DropWeapon,
    /// Оружие отлетает на 1D10 метров    
    ThrowWeapon(i32),
    /// Оружие теряет 1D10 или 1D6 ХП
    WeaponDamaged(i32),
    /// Оружие сломано полностью
    BreakWeapon,
    /// -30% к навыкам на 1D3 раунда
    VisionObscured(i32),
    /// Ударить союзника (Нормальный урон)
    HitAllyNormal,
    /// Ударить союзника (Special урон)
    HitAllySpecial,
    /// Ударить союзника (Critical урон)
    HitAllyCritical,
    /// Ударить себя (Natural weapons)
    HitSelfNormal,
    /// Растяжение: потерять 1 ХП в бьющей конечности
    StrainLimb,
    /// Подвернуть ногу (упасть + штраф к MOV) на n ходов
    TwistAnkle(i32),
}

/// Бросает кубик по нужной таблице. Возвращает Vec, так как бросок 99 и 00
/// вызывает каскад (2 или 3 дополнительных броска!).
pub fn roll_fumble<R: Rng + ?Sized>(table: FumbleTable, rng: &mut R) -> Vec<FumbleEffect> {
    let mut effects = Vec::new();
    let mut rolls_left = 1;

    while rolls_left > 0 {
        rolls_left -= 1;
        let roll = rng.random_range(1..=100);

        match table {
            FumbleTable::MeleeAttack => match roll {
                1..=15 => effects.push(FumbleEffect::LoseNextRound),
                16..=25 => effects.push(FumbleEffect::LoseRounds(rng.random_range(1..=3))),
                26..=40 => effects.push(FumbleEffect::FallProne),
                41..=50 => effects.push(FumbleEffect::DropWeapon),
                51..=60 => effects.push(FumbleEffect::ThrowWeapon(rng.random_range(1..=10))),
                61..=65 => effects.push(FumbleEffect::WeaponDamaged(rng.random_range(1..=10))),
                66..=75 => effects.push(FumbleEffect::VisionObscured(rng.random_range(1..=3))),
                76..=85 => effects.push(FumbleEffect::HitAllyNormal),
                86..=90 => effects.push(FumbleEffect::HitAllySpecial),
                91..=98 => effects.push(FumbleEffect::HitAllyCritical),
                99 => rolls_left += 2,
                100 => rolls_left += 3,
                _ => {}
            },
            FumbleTable::MeleeParry => match roll {
                1..=20 => effects.push(FumbleEffect::LoseNextRound),
                21..=40 => effects.push(FumbleEffect::FallProne),
                41..=50 => effects.push(FumbleEffect::DropWeapon),
                51..=60 => effects.push(FumbleEffect::ThrowWeapon(rng.random_range(1..=10))),
                61..=75 => effects.push(FumbleEffect::VisionObscured(rng.random_range(1..=3))),
                76..=85 => effects.push(FumbleEffect::HitAllyNormal), // Wide open: foe hits normally (в контексте движка можно трактовать так же)
                86..=90 => effects.push(FumbleEffect::HitAllySpecial),
                91..=93 => effects.push(FumbleEffect::HitAllyCritical),
                94..=98 => rolls_left += 2,
                99..=100 => rolls_left += 3,
                _ => {}
            },
            FumbleTable::MissileAttack => match roll {
                1..=15 => effects.push(FumbleEffect::LoseNextRound),
                16..=25 => effects.push(FumbleEffect::LoseRounds(rng.random_range(1..=3))),
                26..=40 => effects.push(FumbleEffect::FallProne),
                41..=55 => effects.push(FumbleEffect::VisionObscured(rng.random_range(1..=3))),
                56..=65 => effects.push(FumbleEffect::ThrowWeapon(rng.random_range(1..=5))),
                66..=80 => effects.push(FumbleEffect::WeaponDamaged(rng.random_range(1..=6))),
                81..=85 => effects.push(FumbleEffect::BreakWeapon),
                86..=90 => effects.push(FumbleEffect::HitAllyNormal),
                91..=95 => effects.push(FumbleEffect::HitAllySpecial),
                96..=98 => effects.push(FumbleEffect::HitAllyCritical),
                99 => rolls_left += 2,
                100 => rolls_left += 3,
                _ => {}
            },

            FumbleTable::NaturalWeapon => match roll {
                1..=25 => effects.push(FumbleEffect::LoseNextRound),
                26..=30 => effects.push(FumbleEffect::LoseRounds(rng.random_range(1..=3))),
                31..=50 => effects.push(FumbleEffect::FallProne),
                51..=60 => {
                    effects.push(FumbleEffect::FallProne);
                    effects.push(FumbleEffect::TwistAnkle(rng.random_range(1..=10)));
                }
                61..=75 => effects.push(FumbleEffect::VisionObscured(rng.random_range(1..=3))),
                76..=85 => effects.push(FumbleEffect::StrainLimb),
                86..=90 => effects.push(FumbleEffect::HitAllyNormal),
                91..=94 => effects.push(FumbleEffect::HitAllySpecial),
                95..=98 => effects.push(FumbleEffect::HitSelfNormal),
                99 => rolls_left += 2,
                100 => rolls_left += 3,
                _ => {} // На всякий случай
            },
        }
    }

    effects
}
