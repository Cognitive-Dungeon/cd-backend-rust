use crate::tile::Tile;
use serde::{ Serialize, Deserialize };
use serde_big_array::BigArray;

// Константы размера (Compile-time)
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

/// Чанк 16x16.
/// Хранится как плоский массив для максимальной скорости доступа (L1 cache friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    // Box<[]> используется, чтобы не забивать стек (256 * 4 байта = 1KB, терпимо, но лучше в куче для больших карт)
    // Но для начала используем простой массив, Rust компилятор умный.
    #[serde(with = "BigArray")]
    tiles: [Tile; CHUNK_AREA],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            tiles: [Tile::VOID; CHUNK_AREA],
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn index(local_x: usize, local_y: usize) -> usize {
        // Убираем проверки границ в Release (unsafe), но пока оставим безопасный вариант
        // Битовый сдвиг вместо умножения: (y * 16) + x
        (local_y << 4) | local_x
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE {
            return None;
        }
        Some(&self.tiles[Self::index(x, y)])
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            self.tiles[Self::index(x, y)] = tile;
        }
    }
}