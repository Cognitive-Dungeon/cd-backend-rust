use crate::sparse_chunk::SparseChunk;
use crate::{CHUNK_SHIFT, Chunk, Tile};
use ahash::{HashMap, HashMapExt};
use cd_core::WorldPos;
use std::sync::RwLock;

pub struct Shard {
    // RwLock внутри шарда защищает только данные этого шарда
    deltas: RwLock<HashMap<WorldPos, SparseChunk>>,
}

impl Shard {
    pub(crate) fn new() -> Self {
        Self {
            deltas: RwLock::new(HashMap::new()),
        }
    }

    // Возвращает копию тайла, если он есть в дельте
    pub(crate) fn get_tile(&self, chunk_key: WorldPos, lx: usize, ly: usize) -> Option<Tile> {
        let guard = self.deltas.read().unwrap();
        if let Some(delta) = guard.get(&chunk_key) {
            return delta.get(lx, ly);
        }
        None
    }

    // Быстрая проверка флага через маску дельты
    pub(crate) fn check_flag_fast(
        &self,
        chunk_key: WorldPos,
        lx: usize,
        ly: usize,
        check_opaque: bool,
    ) -> Option<bool> {
        let guard = self.deltas.read().unwrap();
        if let Some(delta) = guard.get(&chunk_key) {
            let idx = (ly << CHUNK_SHIFT) | lx;
            return Some(if check_opaque {
                delta.opaque_mask.get(idx)
            } else {
                delta.solid_mask.get(idx)
            });
        }
        None
    }

    // Получить блокировку на запись для конкретного чанка
    // Примечание: Это блокирует ВЕСЬ шард на запись.
    // В высоконагруженной системе здесь можно использовать DashMap.
    pub(crate) fn set_tile(
        &self,
        chunk_key: WorldPos,
        lx: usize,
        ly: usize,
        tile: Tile,
        base_chunk: Option<&Chunk>,
    ) {
        let mut guard = self.deltas.write().unwrap();

        let delta = guard.entry(chunk_key).or_insert_with(SparseChunk::new);

        // Lazy initialization масок, если дельта была пустой
        if delta.modifications.is_empty() {
            delta.update_masks(base_chunk);
        }

        delta.set(lx, ly, tile);
    }
}
