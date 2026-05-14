use bevy_ecs::world::Mut;
use cd_common::Glyph;
use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::{Guid, Name, Position, Render, Stats};
use cd_map::{Chunk, Tile};

use crate::{
    Engine,
    world::resources::{DefsCache, MapResource},
};

pub struct WorldGenerator;

impl WorldGenerator {
    /// Генерирует простую тестовую комнату 10x10 в чанке (0,0)
    pub fn generate_test_room(map: &mut MapResource, defs: &DefsCache) {
        let mut chunk = Chunk::new();

        // 1. Находим нужные материалы в кэше по их слагу (slug)
        let floor_mat = defs
            .slug_to_material
            .get("floor_stone")
            .and_then(|id| defs.materials.get(id))
            .expect("Missing floor_stone in CDB!");
        let wall_mat = defs
            .slug_to_material
            .get("wall_stone")
            .and_then(|id| defs.materials.get(id))
            .expect("Missing wall_stone in CDB!");

        // 2. Создаем из них тайлы (Tile) для карты
        let t_floor = Tile {
            material: floor_mat.id,
            flags: floor_mat.flags(),
            variant: 0,
        };

        let t_wall = Tile {
            material: wall_mat.id,
            flags: wall_mat.flags(),
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

    /// Спавнит тестового моба прямо в ECS.
    /// Временный метод — уберём когда появится нормальная система спавна.
    pub fn spawn_test_mob(world: &mut bevy_ecs::world::World) {
        let guid = ObjectGuid::new(1, 2, 1, 9999); // фиксированный тестовый guid
        let pos = WorldPos::new(3, 3, 0); // внутри тестовой комнаты

        let entity = world
            .spawn((
                Guid(guid),
                Position(pos),
                Name("Test Goblin".to_string()),
                Render {
                    glyph: Glyph::new(0x00FF00, b'g'),
                },
                Stats {
                    hp: 30,
                    max_hp: 30,
                    mana: 0,
                    max_mana: 0,
                },
            ))
            .id();

        if let Some(mut registry) =
            world.get_resource_mut::<crate::world::resources::RegistryResource>()
        {
            registry.inner.register(guid, entity);
        }
        if let Some(mut grid) = world.get_resource_mut::<crate::world::resources::GridResource>() {
            grid.inner.insert(guid, pos);
        }

        tracing::info!("Test goblin spawned at {:?}", pos);
    }
}

impl Engine {
    /// Генерирует тестовый мир. Временный метод — уберём когда появится
    /// нормальная система загрузки/генерации карт.
    pub fn generate_test_world(&mut self) {
        self.world
            .resource_scope(|world, mut map: Mut<MapResource>| {
                let defs = world
                    .get_resource::<DefsCache>()
                    .expect("DefsCache must be initialized before generate_test_world");
                crate::world::generator::WorldGenerator::generate_test_room(&mut map, defs);
            });
    }

    pub fn spawn_test_mob(&mut self) {
        crate::world::generator::WorldGenerator::spawn_test_mob(&mut self.world);
    }
}
