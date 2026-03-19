use bevy_ecs::prelude::*;
use cd_common::Glyph;
use cd_ecs::components::{Position, Render};

use crate::{
    EntitySnapshot,
    world::{resources::MapResource, snapshot::ChunkSnapshot},
};

pub struct SnapshotBuilder;

impl SnapshotBuilder {
    /// Строит плоский список сущностей для передачи по сети.
    /// Требует `&mut World`, так как Bevy кэширует стейт запроса внутри.
    pub fn build_entities(world: &mut World) -> Vec<EntitySnapshot> {
        let mut snapshots = Vec::new();
        // Запрашиваем нужные компоненты
        let mut query = world.query::<(&cd_ecs::Guid, &Position, &Render)>();

        for (guid, pos, render) in query.iter(world) {
            snapshots.push(EntitySnapshot {
                guid: Some(guid.0),
                x: pos.0.x(),
                y: pos.0.y(),
                glyph: render.glyph,
            });
        }

        snapshots
    }

    /// Строит снапшот чанка карты.
    /// Здесь достаточно `&World`, так как мы просто достаем ресурс.
    pub fn build_chunk(world: &World, chunk_key: cd_core::WorldPos) -> ChunkSnapshot {
        // Достаем карту из ресурсов мира
        let map_res = world
            .get_resource::<MapResource>()
            .expect("MapRes is missing in the world");

        let mut palette = Vec::new();
        let mut indices = Vec::with_capacity(256);
        let mut mat_to_pal = std::collections::HashMap::new();

        for ly in 0..16 {
            for lx in 0..16 {
                let pos =
                    cd_core::WorldPos::new(chunk_key.x() * 16 + lx, chunk_key.y() * 16 + ly, 0);

                let tile = map_res.inner.get_tile(pos);

                let glyph = match tile.material {
                    0 => Glyph::new(0x000000, b' '), // Пустота
                    1 => Glyph::new(0x555555, b'#'), // Стена
                    2 => Glyph::new(0x222222, b'.'), // Пол
                    _ => Glyph::new(0xFF00FF, b'?'), // Неизвестно
                };

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
