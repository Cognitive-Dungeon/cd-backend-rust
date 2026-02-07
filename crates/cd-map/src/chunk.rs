use crate::tile::Tile;
use crate::bitmask::BitMask256;
use crate::{TileFlags, CHUNK_AREA, CHUNK_SHIFT, CHUNK_SIZE};



/// Чанк 16x16.
/// Хранится как плоский массив для максимальной скорости доступа (L1 cache friendly).
#[derive(Debug, Clone)]
pub struct Chunk {
    indices: Box<[u8; CHUNK_AREA]>,
    palette: Vec<Tile>, // Max 256 elements
    pub(crate) solid_mask: BitMask256,
    pub(crate) opaque_mask: BitMask256,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            indices: Box::new([0; CHUNK_AREA]),
            // Палитра всегда содержит пустой тайл под индексом 0
            palette: vec![Tile::default()],
            solid_mask: BitMask256::default(),
            opaque_mask: BitMask256::default(),
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    /// Установка тайла (только для генерации, не thread-safe для рантайма)
    pub fn set_tile(&mut self, lx: usize, ly: usize, tile: Tile) -> bool {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize {
            return false;
        }

        // 1. Поиск в палитре
        let packed = tile.pack();
        let mut pal_idx = self.palette.iter().position(|t| t.pack() == packed);

        // 2. Добавление, если нет
        if pal_idx.is_none() {
            if self.palette.len() >= 256 {
                return false; // Палитра переполнена
            }
            self.palette.push(tile);
            pal_idx = Some(self.palette.len() - 1);
        }

        let idx = pal_idx.unwrap() as u8;
        let flat_idx = (ly << CHUNK_SHIFT) | lx;

        // 3. Запись индекса
        self.indices[flat_idx] = idx;

        // 4. Обновление масок
        self.update_single_mask_bit(flat_idx, &tile);

        true
    }

    pub fn get_tile(&self, lx: usize, ly: usize) -> Option<Tile> {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize {
            return None;
        }
        let idx = self.indices[(ly << CHUNK_SHIFT) | lx] as usize;
        // Safety: indices всегда указывают на валидный элемент палитры по инварианту
        Some(self.palette[idx])
    }

    pub fn is_solid_local(&self, lx: usize, ly: usize) -> bool {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize { return false; }
        self.solid_mask.get((ly << CHUNK_SHIFT) | lx)
    }

    pub fn is_opaque_local(&self, lx: usize, ly: usize) -> bool {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize { return false; }
        self.opaque_mask.get((ly << CHUNK_SHIFT) | lx)
    }

    pub fn rebuild_masks(&mut self) {
        self.solid_mask = BitMask256::default();
        self.opaque_mask = BitMask256::default();

        // Предварительный кэш свойств палитры
        let props: Vec<(bool, bool)> = self.palette.iter().map(|t| {
            (t.flags.contains(TileFlags::SOLID), t.flags.contains(TileFlags::OPAQUE))
        }).collect();

        for (i, &pal_idx) in self.indices.iter().enumerate() {
            let (is_solid, is_opaque) = props[pal_idx as usize];
            if is_solid { self.solid_mask.set(i, true); }
            if is_opaque { self.opaque_mask.set(i, true); }
        }
    }

    fn update_single_mask_bit(&mut self, idx: usize, tile: &Tile) {
        self.solid_mask.set(idx, tile.flags.contains(TileFlags::SOLID));
        self.opaque_mask.set(idx, tile.flags.contains(TileFlags::OPAQUE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmask_operations() {
        let mut mask = BitMask256::default();

        // Проверяем установку битов
        mask.set(0, true);   // Первый бит
        mask.set(63, true);  // Последний бит первого u64
        mask.set(64, true);  // Первый бит второго u64
        mask.set(255, true); // Самый последний бит

        assert!(mask.get(0));
        assert!(mask.get(63));
        assert!(mask.get(64));
        assert!(mask.get(255));

        assert!(!mask.get(1)); // Неустановленный

        // Снятие флага
        mask.set(0, false);
        assert!(!mask.get(0));
    }

    #[test]
    fn test_chunk_palette_logic() {
        let mut chunk = Chunk::new();

        let t1 = Tile { material: 1, flags: TileFlags::SOLID, variant: 0 };
        let t2 = Tile { material: 2, flags: TileFlags::NONE, variant: 0 };

        // 1. Установка нового тайла
        chunk.set_tile(0, 0, t1);
        assert_eq!(chunk.get_tile(0, 0), Some(t1));
        assert_eq!(chunk.palette.len(), 2); // Void + t1

        // 2. Дедупликация (тот же тайл в другом месте)
        chunk.set_tile(1, 1, t1);
        assert_eq!(chunk.palette.len(), 2); // Палитра не должна вырасти

        // 3. Другой тайл
        chunk.set_tile(2, 2, t2);
        assert_eq!(chunk.palette.len(), 3);
    }

    #[test]
    fn test_chunk_masks_auto_update() {
        let mut chunk = Chunk::new();
        let solid = Tile { material: 1, flags: TileFlags::SOLID, variant: 0 };

        assert!(!chunk.is_solid_local(5, 5));

        chunk.set_tile(5, 5, solid);

        // Маска должна обновиться мгновенно
        assert!(chunk.is_solid_local(5, 5));
    }
}
