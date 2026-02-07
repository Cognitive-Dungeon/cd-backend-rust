use bitflags::bitflags;
use serde::{Deserialize, Serialize};

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

// ID материала (как в Go MaterialID uint16)
pub type MaterialID = u16;

/// Базовая единица карты.
/// Стараемся уложить в 4 байта: 2 (Mat) + 1 (Flags) + 1 (Variant).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)] // Гарантирует layout как в C (поля по порядку)
pub struct Tile {
    pub material: MaterialID,
    pub flags: TileFlags,
    pub variant: u8,
}

impl Tile {
    pub fn is_empty(&self) -> bool {
        self.material == 0
    }

    #[inline(always)]
    pub const fn pack(self) -> u32 {
        ((self.variant as u32) << 24) | ((self.flags.bits() as u32) << 16) | (self.material as u32)
    }

    #[inline(always)]
    pub const fn unpack(packed: u32) -> Self {
        Self {
            material: (packed & 0xFFFF) as u16,
            flags: TileFlags::from_bits_truncate(((packed >> 16) & 0xFF) as u8),
            variant: ((packed >> 24) & 0xFF) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_flags() {
        let mut f = TileFlags::NONE;
        assert!(!f.contains(TileFlags::SOLID));

        f |= TileFlags::SOLID;
        assert!(f.contains(TileFlags::SOLID));

        f.remove(TileFlags::SOLID);
        assert!(!f.contains(TileFlags::SOLID));
    }

    #[test]
    fn test_tile_packing() {
        let original = Tile {
            material: 123,
            flags: TileFlags::SOLID | TileFlags::OPAQUE,
            variant: 42,
        };

        let packed = original.pack();

        // Ручная проверка битов:
        // Variant (42) << 24
        // Flags (3) << 16
        // Material (123)
        let expected = (42 << 24) | (3 << 16) | 123;
        assert_eq!(packed, expected);

        let unpacked = Tile::unpack(packed);
        assert_eq!(original, unpacked);
    }
}