use bevy::ecs::prelude::*;
use cd_core::Glyph;
use cd_ecs::{
    InstanceId, Stats,
    components::{Position, Render},
};

use crate::snapshot::{ChunkSnapshot, EntitySnapshot};
use cd_engine::world::resources::{DefsCache, MapResource};

pub struct SnapshotBuilder;

impl SnapshotBuilder {
    /// Строит плоский список сущностей для передачи по сети.
    /// Требует `&mut World`, так как Bevy кэширует стейт запроса внутри.
    pub fn build_entities(world: &mut World, target_instance: InstanceId) -> Vec<EntitySnapshot> {
        let mut snapshots = Vec::new();
        // Запрашиваем нужные компоненты
        let mut query = world.query::<(&cd_ecs::Guid, &Position, &Render, &Stats, &InstanceId)>();

        for (guid, pos, render, stats, instance) in query.iter(world) {
            // Игнорируем существ из других инстансов
            if *instance != target_instance {
                continue;
            }
            snapshots.push(EntitySnapshot {
                guid: Some(guid.0),
                x: pos.0.x(),
                y: pos.0.y(),
                glyph: render.glyph,
                hp: stats.hp,
                max_hp: stats.max_hp,
            });
        }

        snapshots
    }

    /// Строит снапшот чанка карты для конкретного инстанса.
    /// Здесь достаточно `&World`, так как мы просто достаем ресурс.
    pub fn build_chunk(
        world: &World,
        target_instance: InstanceId,
        chunk_key: cd_core::WorldPos,
    ) -> ChunkSnapshot {
        // Достаем карту из ресурсов мира
        let map_res = world
            .get_resource::<MapResource>()
            .expect("MapRes is missing in the world");
        let defs = world.get_resource::<DefsCache>().unwrap();

        // 1. Пытаемся получить карту конкретного этажа
        let map_opt = map_res.get_map(target_instance);

        let mut palette = Vec::new();
        let mut indices = Vec::with_capacity(256);
        let mut mat_to_pal = std::collections::HashMap::new();

        let mut id_to_glyph = std::collections::HashMap::new();
        for mat in defs.materials.values() {
            id_to_glyph.insert(mat.id, mat.glyph);
        }

        for ly in 0..16 {
            for lx in 0..16 {
                let pos =
                    cd_core::WorldPos::new(chunk_key.x() * 16 + lx, chunk_key.y() * 16 + ly, 0);

                let tile = map_opt.map(|m| m.get_tile(pos)).unwrap_or_default();

                let glyph = id_to_glyph
                    .get(&tile.material)
                    .copied()
                    .unwrap_or(Glyph::new(0x000000, b' ')); // Дефолт (Void)

                let pal_idx = *mat_to_pal.entry(tile.material).or_insert_with(|| {
                    let idx = palette.len() as u8;
                    palette.push(glyph);
                    idx
                });

                indices.push(pal_idx);
            }
        }

        ChunkSnapshot {
            chunk_x: chunk_key.x(),
            chunk_y: chunk_key.y(),
            palette,
            indices,
        }
    }
}
