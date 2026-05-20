use crate::{Cha, Con, Dex, HitPoints, Int, Pow, Siz, Stat, Str};

// domain/chars.rs
/// Контейнер всех базовых статов сущности.
pub struct CharacteristicBlock {
    pub str: Stat<Str>,
    pub con: Stat<Con>,
    pub siz: Stat<Siz>,
    pub int: Stat<Int>,
    pub pow: Stat<Pow>,
    pub dex: Stat<Dex>,
    pub cha: Stat<Cha>,
}

impl CharacteristicBlock {
    // /// Производные параметры считаются строго из базовых
    // pub fn damage_modifier(&self) -> DamageModifier { /* ... */
    // }
    // pub fn max_hit_points(&self) -> HitPoints { /* ... */
    // }
}
