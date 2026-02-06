use bitflags::bitflags;
use serde::{Deserialize, Serialize};

// ID материала (как в Go MaterialID uint16)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MaterialId(pub u16);

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct TileFlags: u8 {
        const NONE     = 0;
        const SOLID    = 1 << 0; // Стена
        const OPAQUE   = 1 << 1; // Блокирует свет
        const LIQUID   = 1 << 2; // Вода/Лава
        const WALKABLE = 1 << 3; // Пол
    }
}

/// Базовая единица карты.
/// Стараемся уложить в 4 байта: 2 (Mat) + 1 (Flags) + 1 (Variant).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[repr(C)] // Гарантирует layout как в C (поля по порядку)
pub struct Tile {
    pub material: MaterialId,
    pub flags: TileFlags,
    pub variant: u8,
}

impl Tile {
    pub const VOID: Self = Self {
        material: MaterialId(0),
        flags: TileFlags::NONE,
        variant: 0,
    };

    #[inline(always)]
    pub fn is_solid(&self) -> bool {
        self.flags.contains(TileFlags::SOLID)
    }
}