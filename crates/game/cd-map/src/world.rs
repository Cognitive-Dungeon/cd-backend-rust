use std::sync::RwLock;
use ahash::{HashMap, HashMapExt};
use cd_core::WorldPos;
use crate::region::{Region};
use crate::shard::{Shard};
use crate::{Chunk, Tile, REGION_MASK, SHARD_COUNT};

pub struct WorldMap {
    // Статический слой: Регионы
    // Используем RwLock для загрузки/выгрузки регионов
    regions: RwLock<HashMap<WorldPos, Region>>,

    // Динамический слой: Шарды
    // Массив фиксированного размера
    shards: Box<[Shard; SHARD_COUNT]>,

    default_tile: Tile,
}

impl WorldMap {
    pub fn new() -> Self {
        let mut shards_vec = Vec::with_capacity(SHARD_COUNT);
        for _ in 0..SHARD_COUNT {
            shards_vec.push(Shard::new());
        }
        let shards = shards_vec.try_into().ok().expect("Failed to init shards");

        Self {
            regions: RwLock::new(HashMap::new()),
            shards: Box::new(shards),
            default_tile: Tile::default(),
        }
    }

    // --- Public API ---

    pub fn get_tile(&self, pos: WorldPos) -> Tile {
        let chunk_key = pos.chunk_key();
        let (lx, ly) = pos.local_coords();

        // 1. Dynamic Layer
        let shard = &self.shards[chunk_key.shard_index()];
        if let Some(tile) = shard.get_tile(chunk_key, lx, ly) {
            return tile;
        }

        // 2. Static Layer
        self.get_static_tile(chunk_key, lx, ly).unwrap_or(self.default_tile)
    }

    pub fn is_solid_fast(&self, pos: WorldPos) -> bool {
        self.check_flag_fast(pos, false)
    }

    pub fn is_opaque_fast(&self, pos: WorldPos) -> bool {
        self.check_flag_fast(pos, true)
    }

    fn check_flag_fast(&self, pos: WorldPos, check_opaque: bool) -> bool {
        let chunk_key = pos.chunk_key();
        let (lx, ly) = pos.local_coords();

        let shard = &self.shards[chunk_key.shard_index()];
        if let Some(val) = shard.check_flag_fast(chunk_key, lx, ly, check_opaque) {
            return val;
        }

        let regions = self.regions.read().unwrap();
        let region_key = chunk_key.region_key();

        if let Some(region) = regions.get(&region_key) {
            let (cx, cy, _) = chunk_key.xyz();
            let rx = (cx & REGION_MASK) as usize;
            let ry = (cy & REGION_MASK) as usize;

            if let Some(chunk) = region.get_chunk(rx, ry) {
                return if check_opaque { chunk.is_opaque_local(lx, ly) } else { chunk.is_solid_local(lx, ly) };
            }
        }
        false
    }

    pub fn set_tile(&self, pos: WorldPos, tile: Tile) {
        let chunk_key = pos.chunk_key();
        let (lx, ly) = pos.local_coords();
        let shard = &self.shards[chunk_key.shard_index()];

        let regions_guard = self.regions.read().unwrap();
        let region_key = chunk_key.region_key();
        // Получаем Snapshot базы для инициализации масок в дельте
        let base_chunk = regions_guard.get(&region_key).and_then(|r| {
            let (cx, cy, _) = chunk_key.xyz();
            let rx = (cx & REGION_MASK) as usize;
            let ry = (cy & REGION_MASK) as usize;
            r.get_chunk(rx, ry)
        });

        shard.set_tile(chunk_key, lx, ly, tile, base_chunk);
    }

    pub fn put_chunk(&self, chunk_key: WorldPos, chunk: Chunk) {
        let region_key = chunk_key.region_key();
        let (cx, cy, _) = chunk_key.xyz();
        let rx = (cx & REGION_MASK) as usize;
        let ry = (cy & REGION_MASK) as usize;

        let mut regions = self.regions.write().unwrap();
        let region = regions.entry(region_key).or_insert_with(Region::new);
        let dest_chunk = region.get_or_create_chunk(rx, ry);
        *dest_chunk = chunk;
    }

    // --- Private Helpers ---

    fn get_static_tile(&self, chunk_key: WorldPos, lx: usize, ly: usize) -> Option<Tile> {
        let regions = self.regions.read().unwrap();
        let region_key = chunk_key.region_key();

        if let Some(region) = regions.get(&region_key) {
            let (cx, cy, _) = chunk_key.xyz();
            let rx = (cx & REGION_MASK) as usize;
            let ry = (cy & REGION_MASK) as usize;
            if let Some(chunk) = region.get_chunk(rx, ry) {
                return Some(chunk.get_tile(lx, ly));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::TileFlags;
    use super::*;

    #[test]
    fn test_world_layers_integration() {
        let world = WorldMap::new();
        let pos = WorldPos::new(10, 10, 0);

        let t_static = Tile { material: 1, flags: TileFlags::SOLID, variant: 0 }; // Стена
        let t_dynamic = Tile { material: 2, flags: TileFlags::LIQUID, variant: 0 }; // Вода (разлили поверх)

        // 1. Загружаем статический чанк
        let mut chunk = Chunk::new();
        let (lx, ly) = pos.local_coords();
        chunk.set_tile(lx, ly, t_static);
        world.put_chunk(pos.chunk_key(), chunk);

        // Проверяем, что статика видна
        assert_eq!(world.get_tile(pos).material, t_static.material);
        assert!(world.is_solid_fast(pos));

        // 2. Вносим динамическое изменение (Sparse Layer)
        world.set_tile(pos, t_dynamic);

        // Проверяем, что динамика перекрыла статику
        let current = world.get_tile(pos);
        assert_eq!(current.material, t_dynamic.material);

        // Проверяем быстрые маски
        assert!(!world.is_solid_fast(pos)); // Вода не Solid
        // (предполагаем, что t_dynamic.flags содержат LIQUID, но не SOLID)
    }

    #[test]
    fn test_threading_smoke_test() {
        // Простейший тест на дедлоки (хотя для полноценной проверки нужны потоки)
        use std::sync::Arc;
        use std::thread;

        let world = Arc::new(WorldMap::new());
        let mut handles = vec![];

        for i in 0..10 {
            let w = world.clone();
            handles.push(thread::spawn(move || {
                let p = WorldPos::new(i, 0, 0);
                w.set_tile(p, Tile { material: 1, ..Default::default() });
                w.get_tile(p);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}