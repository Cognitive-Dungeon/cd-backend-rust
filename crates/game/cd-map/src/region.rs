use bitflags::bitflags;
use crate::chunk::Chunk;
use crate::{REGION_AREA, REGION_SHIFT};

bitflags! {
    /// Маска для быстрого определения, загружен ли чанк внутри региона.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct RegionFlags: u8 {
        const MODIFIED = 1 << 0;
    }
}

/// Регион — это крупный статический блок карты.
/// Используется для стриминга с диска.
pub struct Region {
    // Линеаризованный массив чанков.
    chunks: Box<[Chunk; REGION_AREA]>,

    // Битовая маска, указывающая, инициализирован ли чанк реальными данными.
    pub presence_map: [u64; REGION_AREA / 64],
}

impl Default for Region {
    fn default() -> Self {
        let mut vec = Vec::with_capacity(REGION_AREA);
        for _ in 0..REGION_AREA {
            vec.push(Chunk::default());
        }

        // Превращаем Vec<Chunk> в Box<[Chunk]>
        let boxed_slice: Box<[Chunk]> = vec.into_boxed_slice();

        // Превращаем Box<[Chunk]> в Box<[Chunk; 1024]>
        // unsafe здесь нужен, так как try_into для больших массивов может быть не оптимизирован,
        // но standard library `try_into()` работает безопасно.
        let chunks = boxed_slice.try_into().map_err(|_| "Allocation error").unwrap();

        Self {
            chunks,
            presence_map: [0; REGION_AREA / 64],
        }
    }
}

impl Region {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn get_chunk(&self, rx: usize, ry: usize) -> Option<&Chunk> {
        let idx = (ry << REGION_SHIFT) | rx; // ry * 32 + rx

        // Проверяем бит присутствия
        if !self.check_presence(idx) {
            return None;
        }

        // Unsafe get для скорости (idx гарантированно < 1024 из-за маски rx/ry caller'а)
        unsafe { Some(self.chunks.get_unchecked(idx)) }
    }

    #[inline(always)]
    pub fn get_chunk_mut(&mut self, rx: usize, ry: usize) -> Option<&mut Chunk> {
        let idx = (ry << REGION_SHIFT) | rx;
        if !self.check_presence(idx) {
            return None;
        }
        unsafe { Some(self.chunks.get_unchecked_mut(idx)) }
    }

    /// Активирует чанк для записи.
    /// Возвращает мутабельную ссылку.
    pub fn get_or_create_chunk(&mut self, rx: usize, ry: usize) -> &mut Chunk {
        let idx = (ry << REGION_SHIFT) | rx;
        if !self.check_presence(idx) {
            self.set_presence(idx, true);
            // Чанк уже есть в памяти (default), просто помечаем его живым
        }
        unsafe { self.chunks.get_unchecked_mut(idx) }
    }

    #[inline]
    fn check_presence(&self, idx: usize) -> bool {
        let block = idx / 64;
        let bit = 1u64 << (idx % 64);
        unsafe { (*self.presence_map.get_unchecked(block) & bit) != 0 }
    }

    #[inline]
    fn set_presence(&mut self, idx: usize, val: bool) {
        let block = idx / 64;
        let bit = 1u64 << (idx % 64);
        unsafe {
            let p = self.presence_map.get_unchecked_mut(block);
            if val { *p |= bit; } else { *p &= !bit; }
        }
    }
}
