use cd_core::WorldPos;
use cd_map::{Chunk, Tile};

use crate::world::resources::{DefsCache, MapResource};

pub struct WorldGenerator;

impl WorldGenerator {
    /// Генерирует простую тестовую комнату 10x10 в чанке (0,0)
    pub fn generate_test_room(map: &mut MapResource, defs: &DefsCache) {
        let mut chunk = Chunk::new();

        // 1. Находим нужные материалы в кэше по их слагу (slug)
        let floor_mat = defs
            .materials
            .get("floor_stone")
            .expect("Missing floor_stone in CDB!");
        let wall_mat = defs
            .materials
            .get("wall_stone")
            .expect("Missing wall_stone in CDB!");

        // 2. Создаем из них тайлы (Tile) для карты
        let t_floor = Tile {
            material: floor_mat.mat_id,
            flags: floor_mat.flags,
            variant: 0,
        };

        let t_wall = Tile {
            material: wall_mat.mat_id,
            flags: wall_mat.flags,
            variant: 0,
        };

        // 3. Заполняем чанк
        for y in 0..16 {
            for x in 0..16 {
                // Делаем комнату 10x10 с отступом
                if (2..=12).contains(&x) && (2..=12).contains(&y) {
                    if x == 2 || x == 12 || y == 2 || y == 12 {
                        chunk.set_tile(x, y, t_wall); // Стены по краям
                    } else {
                        chunk.set_tile(x, y, t_floor); // Пол внутри
                    }
                }
            }
        }

        // 4. Сохраняем чанк в карту
        map.inner.put_chunk(WorldPos::new(0, 0, 0), chunk);
        tracing::info!("Test room generated using Depot materials!");
    }
}
