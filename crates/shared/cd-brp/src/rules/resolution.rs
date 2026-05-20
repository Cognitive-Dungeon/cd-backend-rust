// rules/resolution.rs

use crate::{CharacteristicMarker, D100Roll, ExchangeOutcome, SkillRating, Stat, SuccessLevel};

pub trait CheckResolver {
    /// Разрешает бросок D100 против рейтинга навыка.
    fn resolve(roll: D100Roll, rating: SkillRating) -> SuccessLevel;
}

pub trait ResistanceResolver {
    /// Сравнивает две характеристики ОДНОГО ТИПА.
    fn resolve<T: CharacteristicMarker>(active: Stat<T>, passive: Stat<T>, roll: D100Roll) -> bool;
}

pub trait MatrixResolver {
    /// Разрешает столкновение в ближнем бою по таблице Attack and Defence Matrix (стр. 51).
    fn resolve_melee(
        attacker: SuccessLevel,
        defender: Option<SuccessLevel>,
        is_dodge: bool,
    ) -> ExchangeOutcome;
}
