use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::ser::SerializeStruct;
use std::fmt;

/// Упакованная координата (X, Y, Z).
/// Layout: [ Z (12) | Y (26) | X (26) ]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPos(u64);

// Константы размеров мира (дублируем логику из map для инкапсуляции)
const CHUNK_SHIFT: i32 = 4;
const CHUNK_MASK: i32 = 15;
const REGION_SHIFT: i32 = 5;
const SHARD_MASK: usize = 64 - 1;

impl WorldPos {
    const BITS_Z: u64 = 12;
    const BITS_Y: u64 = 26;
    const BITS_X: u64 = 26;

    const SHIFT_X: u64 = 0;
    const SHIFT_Y: u64 = Self::BITS_X;
    const SHIFT_Z: u64 = Self::BITS_X + Self::BITS_Y;

    const MASK_X: u64 = (1 << Self::BITS_X) - 1;
    const MASK_Y: u64 = (1 << Self::BITS_Y) - 1;
    const MASK_Z: u64 = (1 << Self::BITS_Z) - 1;

    // Смещения для поддержки отрицательных координат
    const OFFSET_X: i32 = 1 << (Self::BITS_X - 1);
    const OFFSET_Y: i32 = 1 << (Self::BITS_Y - 1);
    const OFFSET_Z: i32 = 1 << (Self::BITS_Z - 1);

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        let ux = (x + Self::OFFSET_X) as u64 & Self::MASK_X;
        let uy = (y + Self::OFFSET_Y) as u64 & Self::MASK_Y;
        let uz = (z + Self::OFFSET_Z) as u64 & Self::MASK_Z;

        Self((uz << Self::SHIFT_Z) | (uy << Self::SHIFT_Y) | ux)
    }

    // --- Accessors ---

    #[inline(always)]
    pub fn x(&self) -> i32 {
        let val = (self.0 >> Self::SHIFT_X) & Self::MASK_X;
        (val as i32) - Self::OFFSET_X
    }

    #[inline(always)]
    pub fn y(&self) -> i32 {
        let val = (self.0 >> Self::SHIFT_Y) & Self::MASK_Y;
        (val as i32) - Self::OFFSET_Y
    }

    #[inline(always)]
    pub fn z(&self) -> i32 {
        let val = (self.0 >> Self::SHIFT_Z) & Self::MASK_Z;
        (val as i32) - Self::OFFSET_Z
    }

    /// Деструктуризация для удобства
    #[inline]
    pub fn xyz(&self) -> (i32, i32, i32) {
        (self.x(), self.y(), self.z())
    }

    // --- WorldMap Keys Helpers ---

    /// Получить ключ чанка (глобальные координаты / 16)
    pub fn chunk_key(&self) -> WorldPos {
        let (x, y, z) = self.xyz();
        WorldPos::new(x >> CHUNK_SHIFT, y >> CHUNK_SHIFT, z)
    }

    /// Получить локальные координаты внутри чанка (0..15)
    pub fn local_coords(&self) -> (usize, usize) {
        let (x, y, _) = self.xyz();
        ((x & CHUNK_MASK) as usize, (y & CHUNK_MASK) as usize)
    }

    /// Получить ключ региона (координаты чанка / 32)
    /// self здесь должен быть уже chunk_key
    pub fn region_key(&self) -> WorldPos {
        let (cx, cy, _) = self.xyz();
        WorldPos::new(cx >> REGION_SHIFT, cy >> REGION_SHIFT, 0)
    }

    /// Индекс шарда (для chunk_key)
    pub fn shard_index(&self) -> usize {
        let (cx, cy, _) = self.xyz();
        ((cx ^ cy) as usize) & SHARD_MASK
    }
}

impl fmt::Debug for WorldPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

// --- Serialization ---

impl Serialize for WorldPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Сериализуем как объект, чтобы в JSON было читаемо {"x":1,"y":2,"z":0}
        // Для бинарных форматов можно было бы сериализовать self.0 напрямую
        let (x, y, z) = self.xyz();
        let mut state = serializer.serialize_struct("WorldPos", 3)?;
        state.serialize_field("x", &x)?;
        state.serialize_field("y", &y)?;
        state.serialize_field("z", &z)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for WorldPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PosDto { x: i32, y: i32, z: i32 }

        let dto = PosDto::deserialize(deserializer)?;
        Ok(WorldPos::new(dto.x, dto.y, dto.z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_packing_unpacking() {
        let cases = vec![
            (0, 0, 0),
            (10, 20, 5),
            (-1, -1, 0),
            (-100, 500, -5),
            (1024, -1024, 10),
        ];

        for (x, y, z) in cases {
            let p = WorldPos::new(x, y, z);
            assert_eq!(p.x(), x, "X mismatch");
            assert_eq!(p.y(), y, "Y mismatch");
            assert_eq!(p.z(), z, "Z mismatch");
        }
    }

    #[test]
    fn test_chunk_keys() {
        // Тест из Go: Pos(17, 0, 5) -> Chunk(1, 0, 5)
        let p = WorldPos::new(17, 0, 5);
        let ck = p.chunk_key();
        assert_eq!(ck.x(), 1);
        assert_eq!(ck.y(), 0);
        assert_eq!(ck.z(), 5);

        // Тест отрицательных координат
        // В Go: -17 -> Chunk -2
        let p_neg = WorldPos::new(-17, 0, 0);
        let ck_neg = p_neg.chunk_key();
        assert_eq!(ck_neg.x(), -2);
    }

    #[test]
    fn test_local_coords() {
        let p = WorldPos::new(17, 0, 0); // 16 + 1
        let (lx, ly) = p.local_coords();
        assert_eq!(lx, 1);
        assert_eq!(ly, 0);

        let p_neg = WorldPos::new(-1, 0, 0); // -1 это 15-й тайл в чанке -1
        let (lx_neg, _) = p_neg.local_coords();
        assert_eq!(lx_neg, 15);
    }

    #[test]
    fn test_region_key() {
        // Чанк 33 находится в регионе 1 (32 чанка на регион)
        let chunk_pos = WorldPos::new(33, 0, 0);
        let reg_key = chunk_pos.region_key();
        assert_eq!(reg_key.x(), 1);
        assert_eq!(reg_key.y(), 0);
    }
}