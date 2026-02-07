use crate::chunk::Chunk;
use crate::{REGION_AREA, REGION_SHIFT};

/// Регион — это крупный статический блок карты.
/// Используется для стриминга с диска.
pub struct Region {
    // Линеаризованный массив чанков.
    // Option - чанк может быть не загружен.
    // Box - чтобы регион не занимал мегабайты на стеке при создании, и для immutable sharing.
    chunks: Vec<Option<Box<Chunk>>>,
}

impl Default for Region {
    fn default() -> Self {
        let mut chunks = Vec::with_capacity(REGION_AREA);
        for _ in 0..REGION_AREA {
            chunks.push(None);
        }
        Self { chunks }
    }
}

impl Region {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_chunk(&self, rx: usize, ry: usize) -> Option<&Chunk> {
        let idx = (ry << REGION_SHIFT) | rx;
        self.chunks.get(idx).and_then(|opt| opt.as_deref())
    }

    pub fn put_chunk(&mut self, rx: usize, ry: usize, chunk: Chunk) {
        let idx = (ry << REGION_SHIFT) | rx;
        if idx < self.chunks.len() {
            self.chunks[idx] = Some(Box::new(chunk));
        }
    }
}
