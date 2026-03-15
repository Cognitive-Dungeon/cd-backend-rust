use ahash::HashMap;
use crate::bitmask::BitMask256;
use crate::{Chunk, Tile, TileFlags, CHUNK_SHIFT};

#[derive(Clone, Debug, Default)]
pub struct SparseChunk {
    // Ключ - упакованный индекс (ly << 4 | lx)
    pub(crate) modifications: HashMap<u8, Tile>,
    pub(crate) solid_mask: BitMask256,
    pub(crate) opaque_mask: BitMask256,
    is_dirty: bool,
}

impl SparseChunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_masks(&mut self, base: Option<&Chunk>) {
        // 1. Копируем базу
        if let Some(b) = base {
            self.solid_mask = b.solid_mask;
            self.opaque_mask = b.opaque_mask;
        } else {
            self.solid_mask = BitMask256::default();
            self.opaque_mask = BitMask256::default();
        }

        // 2. Накатываем изменения
        let solid = &mut self.solid_mask;
        let opaque = &mut self.opaque_mask;

        for (&idx, tile) in &self.modifications {
            // Вызываем статическую функцию, передавая ей только нужные поля
            Self::apply_tile_to_masks(solid, opaque, idx as usize, tile);
        }
    }

    pub fn get(&self, lx: usize, ly: usize) -> Option<Tile> {
        let idx = ((ly << CHUNK_SHIFT) | lx) as u8;
        self.modifications.get(&idx).copied()
    }

    pub fn set(&mut self, lx: usize, ly: usize, tile: Tile) {
        let idx = ((ly << CHUNK_SHIFT) | lx) as u8;
        self.modifications.insert(idx, tile);
        self.is_dirty = true;

        Self::apply_tile_to_masks(&mut self.solid_mask, &mut self.opaque_mask, idx as usize, &tile);
    }

    // --- Helpers ---
    fn apply_tile_to_masks(
        solid_mask: &mut BitMask256,
        opaque_mask: &mut BitMask256,
        idx: usize,
        tile: &Tile
    ) {
        solid_mask.set(idx, tile.flags.contains(TileFlags::SOLID));
        opaque_mask.set(idx, tile.flags.contains(TileFlags::OPAQUE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_masks_hydration() {
        // Создаем "базовый" чанк со стеной в (0,0)
        let mut base = Chunk::new();
        let wall = Tile { material: 1, flags: TileFlags::SOLID, variant: 0 };
        base.set_tile(0, 0, wall);

        // Создаем дельту
        let mut sparse = SparseChunk::new();

        // Инициализируем маски из базы
        sparse.update_masks(Some(&base));
        assert!(sparse.solid_mask.get(0)); // Должно унаследоваться от базы

        // Ломаем стену в дельте (ставим пустой пол)
        let floor = Tile { material: 2, flags: TileFlags::NONE, variant: 0 };
        sparse.set(0, 0, floor); // Overwrite (0,0)

        // Теперь в дельте стены быть не должно
        assert!(!sparse.solid_mask.get(0));
    }
}