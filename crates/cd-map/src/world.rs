use crate::chunk::Chunk;
use cd_core::WorldPos;
use std::collections::HashMap;

#[derive(Default)]
pub struct WorldMap {
    // Ключ: (chunk_x, chunk_y)
    chunks: HashMap<(i32, i32), Chunk>,
}

impl WorldMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Конвертирует мировую позицию в координату чанка
    #[inline]
    fn chunk_key(pos: WorldPos) -> (i32, i32) {
        // Битовый сдвиг на 4 (деление на 16)
        (pos.x() >> 4, pos.y() >> 4)
    }

    /// Конвертирует мировую позицию в локальную внутри чанка
    #[inline]
    fn local_key(pos: WorldPos) -> (usize, usize) {
        // Битовая маска 15 (0xF) == остаток от деления на 16
        ((pos.x() & 15) as usize, (pos.y() & 15) as usize)
    }

    pub fn insert_chunk(&mut self, chunk_x: i32, chunk_y: i32, chunk: Chunk) {
        self.chunks.insert((chunk_x, chunk_y), chunk);
    }

    pub fn get_chunk(&self, x: i32, y: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, y))
    }

    pub fn get_chunk_mut(&mut self, x: i32, y: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(x, y))
    }

    /// Быстрая проверка коллизии (IsSolid)
    pub fn is_solid(&self, pos: WorldPos) -> bool {
        let (cx, cy) = Self::chunk_key(pos);

        if let Some(chunk) = self.chunks.get(&(cx, cy)) {
            let (lx, ly) = Self::local_key(pos);
            // unwrap безопасен, т.к. local_key гарантирует 0..15
            chunk.get(lx, ly).map(|t| t.is_solid()).unwrap_or(false)
        } else {
            false // Если чанка нет - считаем пустотой (или стеной, зависит от дизайна)
        }
    }
}