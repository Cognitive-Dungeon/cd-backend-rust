use ahash::{HashMap, HashMapExt};
use crate::tile::Tile;
use crate::bitmask::BitMask256;
use crate::{TileFlags, CHUNK_AREA, CHUNK_SHIFT, CHUNK_SIZE};

/// Helper для быстрой генерации чанка.
/// Использует HashMap для мгновенного поиска в палитре (O(1)).
/// Используйте его при генерации мира, вместо прямого chunk.set_tile.
pub struct ChunkBuilder {
    chunk: Chunk,
    lut: HashMap<u32, u8>, // Look-Up Table: PackedTile -> Index
}

impl ChunkBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            chunk: Chunk::default(),
            lut: HashMap::new(),
        };
        // 0-й индекс всегда занят пустым тайлом
        builder.lut.insert(0, 0);
        builder
    }

    pub fn set_tile(&mut self, lx: usize, ly: usize, tile: Tile) {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize { return; }

        let packed = tile.pack();

        // 1. Быстрый поиск через HashMap O(1)
        let idx = if let Some(&idx) = self.lut.get(&packed) {
            idx
        } else {
            // 2. Если нет - добавляем в палитру чанка
            let len = self.chunk.palette_len as usize;
            if len >= 256 { return; } // Overflow

            self.chunk.palette[len] = packed;
            self.chunk.palette_len += 1;

            let new_idx = len as u8;
            self.lut.insert(packed, new_idx);
            new_idx
        };

        // 3. Пишем индекс
        let flat_idx = (ly << 4) | lx;
        self.chunk.indices[flat_idx] = idx;

        // 4. Обновляем маски (можно отложить до build(), но сделаем сразу)
        // Тут дублируется логика масок, но для билдера это нормально
        let flags = (packed >> 16) as u8;
        unsafe {
            // Используем внутренний метод без проверок, т.к. flat_idx валиден
            // Мы дублируем логику BitMask::set для инлайнинга и скорости
            let block = flat_idx >> 6;
            let bit = 1u64 << (flat_idx & 63);

            let solid_ptr = self.chunk.solid_mask.data.get_unchecked_mut(block);
            if (flags & TileFlags::SOLID.bits()) != 0 { *solid_ptr |= bit; } else { *solid_ptr &= !bit; }

            let opaque_ptr = self.chunk.opaque_mask.data.get_unchecked_mut(block);
            if (flags & TileFlags::OPAQUE.bits()) != 0 { *opaque_ptr |= bit; } else { *opaque_ptr &= !bit; }
        }

    }

    /// Превращает билдер в готовый Chunk, пересчитывая маски напоследок для гарантии.
    pub fn build(mut self) -> Chunk {
        self.chunk.rebuild_masks();
        self.chunk
    }
}

/// Чанк 16x16.
/// Хранится как плоский массив для максимальной скорости доступа (L1 cache friendly).
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct Chunk {
    // Indices теперь просто массив байт внутри структуры
    pub indices: [u8; CHUNK_AREA],

    // Palette хранит упакованные u32 тайлы.
    // Это быстрее, чем Vec<Tile>, и не требует аллокаций.
    pub palette: [u32; 256],
    pub palette_len: u8,

    pub solid_mask: BitMask256,
    pub opaque_mask: BitMask256,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            indices: [0; CHUNK_AREA],
            palette: [0; 256], // Забито нулями (Void tile)
            palette_len: 1,    // 0-й индекс всегда занят Void
            solid_mask: BitMask256::default(),
            opaque_mask: BitMask256::default(),
        }
    }
}

#[allow(unsafe_code)]
impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    /// Установка тайла (только для генерации, не thread-safe для рантайма)
    pub fn set_tile(&mut self, lx: usize, ly: usize, tile: Tile) -> bool {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize { return false; }

        let packed = tile.pack();
        let flat_idx = (ly << CHUNK_SHIFT) | lx;

        // 1. Быстрый поиск в палитре (Linear scan по u32 очень быстр для N < 20)
        // Для N=256 SIMD был бы лучше, но обычно палитра маленькая.
        // Критика советовала LUT. Если генерация будет тормозить, добавим builder с LUT.
        // Сейчас хранить LUT внутри чанка слишком дорого по памяти.
        let mut pal_idx = None;

        let len = self.palette_len as usize;
        for i in 0..len {
            unsafe {
                if *self.palette.get_unchecked(i) == packed {
                    pal_idx = Some(i as u8);
                    break;
                }
            }
        }

        let idx = match pal_idx {
            Some(i) => i,
            None => {
                if len >= CHUNK_AREA { return false; } // Палитра переполнена
                self.palette[len] = packed;
                self.palette_len += 1;
                len as u8
            }
        };

        // 2. Запись индекса
        self.indices[flat_idx] = idx;

        // 3. Обновление масок (Inlined logic)
        // Нам не нужно распаковывать весь тайл, достаточно проверить биты флагов в u32
        // Layout: [Variant | Flags | Material]
        // Flags находятся в байте 1 (сдвиг 16).
        // 0x01 = SOLID, 0x02 = OPAQUE
        let flags = (packed >> 16) as u8;

        unsafe {
            // Используем внутренний метод без проверок, т.к. flat_idx валиден
            // Мы дублируем логику BitMask::set для инлайнинга и скорости
            let block = flat_idx >> 6;
            let bit = 1u64 << (flat_idx & 63);

            let solid_ptr = self.solid_mask.data.get_unchecked_mut(block);
            if (flags & TileFlags::SOLID.bits()) != 0 { *solid_ptr |= bit; } else { *solid_ptr &= !bit; }

            let opaque_ptr = self.opaque_mask.data.get_unchecked_mut(block);
            if (flags & TileFlags::OPAQUE.bits()) != 0 { *opaque_ptr |= bit; } else { *opaque_ptr &= !bit; }
        }

        true
    }

    #[inline(always)]
    pub fn get_tile(&self, lx: usize, ly: usize) -> Tile {
        if lx >= CHUNK_SIZE as usize || ly >= CHUNK_SIZE as usize {
            return Tile::default();
        }

        unsafe {
            // 1. Получаем индекс из массива индексов
            let flat_idx = (ly << CHUNK_SHIFT) | lx;
            let pal_idx = *self.indices.get_unchecked(flat_idx);

            // 2. Получаем упакованный тайл из палитры
            let packed = *self.palette.get_unchecked(pal_idx as usize);

            // 3. Распаковываем (бесплатно, это просто cast)
            Tile::unpack(packed)
        }
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

        // Создаем LUT свойств на стеке (Stack allocation), а не Vector
        let mut props = [(false, false); 256];
        let len = self.palette_len as usize;

        for i in 0..len {
            let packed = self.palette[i];
            let flags = (packed >> 16) as u8;
            props[i] = (
                (flags & TileFlags::SOLID.bits()) != 0,
                (flags & TileFlags::OPAQUE.bits()) != 0
            );
        }

        for i in 0..CHUNK_AREA {
            let pal_idx = self.indices[i] as usize;
            let (s, o) = unsafe { *props.get_unchecked(pal_idx) }; // pal_idx < len всегда
            if s { self.solid_mask.set(i, true); }
            if o { self.opaque_mask.set(i, true); }
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
        assert_eq!(chunk.get_tile(0, 0), t1);
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
