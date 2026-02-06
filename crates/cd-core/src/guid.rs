use serde::{Deserialize, Serialize};
use std::fmt;

/// Строгий тип для идентификатора объекта.
/// Битовая раскладка (64 бита):
/// [ Shard (8) | Type (8) | Generation (16) | Index (32) ]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct ObjectGuid(u64);

impl ObjectGuid {
    // Константы битовых сдвигов (Compile-time constants)
    const BITS_INDEX: u64 = 32;
    const BITS_GEN: u64 = 16;
    const BITS_TYPE: u64 = 8;
    const BITS_SHARD: u64 = 8;

    const SHIFT_GEN: u64 = Self::BITS_INDEX;
    const SHIFT_TYPE: u64 = Self::BITS_INDEX + Self::BITS_GEN;
    const SHIFT_SHARD: u64 = Self::BITS_INDEX + Self::BITS_GEN + Self::BITS_TYPE;

    const MASK_INDEX: u64 = (1 << Self::BITS_INDEX) - 1;
    const MASK_GEN: u64 = (1 << Self::BITS_GEN) - 1;
    const MASK_TYPE: u64 = (1 << Self::BITS_TYPE) - 1;
    const MASK_SHARD: u64 = (1 << Self::BITS_SHARD) - 1;

    pub const NIL: ObjectGuid = ObjectGuid(0);

    /// Создает новый GUID.
    /// Panic: в debug режиме проверит переполнение, в release обрежет,
    /// но лучше использовать try_new для валидации.
    #[inline]
    pub fn new(shard: u8, type_id: u8, generation: u16, index: u32) -> Self {
        let raw = ((shard as u64) << Self::SHIFT_SHARD)
            | ((type_id as u64) << Self::SHIFT_TYPE)
            | ((generation as u64) << Self::SHIFT_GEN)
            | (index as u64);
        Self(raw)
    }

    #[inline]
    pub fn index(&self) -> u32 {
        (self.0 & Self::MASK_INDEX) as u32
    }

    #[inline]
    pub fn generation(&self) -> u16 {
        ((self.0 >> Self::SHIFT_GEN) & Self::MASK_GEN) as u16
    }

    #[inline]
    pub fn type_id(&self) -> u8 {
        ((self.0 >> Self::SHIFT_TYPE) & Self::MASK_TYPE) as u8
    }

    #[inline]
    pub fn shard_id(&self) -> u8 {
        ((self.0 >> Self::SHIFT_SHARD) & Self::MASK_SHARD) as u8
    }

    pub fn is_nil(&self) -> bool {
        self.0 == 0
    }

    /// Получить "сырое" значение (для БД или Сети)
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Сериализация в JSON как число (или строка, если нужно для JS)
impl Serialize for ObjectGuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for ObjectGuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let val = s.parse::<u64>().map_err(serde::de::Error::custom)?;
        Ok(Self(val))
    }
}

impl fmt::Debug for ObjectGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Guid(ShardID:{}, TypeID:{}, Generation:{}, Index:{})",
            self.shard_id(),
            self.type_id(),
            self.generation(),
            self.index()
        )
    }
}

impl fmt::Display for ObjectGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}