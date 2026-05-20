use serde::{Deserialize, Serialize};

use crate::{Cha, Con, Dex, Edu, Int, Pow, Siz, Stat, Str};

// domain/chars.rs
/// Контейнер всех базовых статов сущности.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacteristicBlock {
    pub str: Stat<Str>,
    pub con: Stat<Con>,
    pub siz: Stat<Siz>,
    pub int: Stat<Int>,
    pub pow: Stat<Pow>,
    pub dex: Stat<Dex>,
    pub cha: Stat<Cha>,
    pub edu: Option<Stat<Edu>>,
}
