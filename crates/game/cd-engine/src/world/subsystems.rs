use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use cd_core::{ObjectGuid, WorldPos};
use cd_map::Tile;

use crate::world::resources::{GridResource, MapResource, RegistryResource};

/// Подсистема Пространства. Прячет внутри работу с картой и сеткой!
#[derive(SystemParam)]
pub struct SpatialSubsystem<'w> {
    pub map: Res<'w, MapResource>,
    pub grid: ResMut<'w, GridResource>,
    pub registry: Res<'w, RegistryResource>,
}

impl<'w> SpatialSubsystem<'w> {
    /// Безопасное перемещение с обновлением всех индексов (без бойлерплейта)
    pub fn move_entity(
        &mut self,
        guid: ObjectGuid,
        from: WorldPos,
        to: WorldPos,
    ) -> Result<(), &'static str> {
        if self.map.inner.is_solid_fast(to) {
            return Err("Solid tile");
        }
        // Всю логику коллизий и обновления SpatialGrid пишем здесь
        self.grid.inner.move_entity(guid, from, to);
        Ok(())
    }

    /// Проверка, кто стоит в тайле
    pub fn get_entities_at(&self, pos: WorldPos) -> &[ObjectGuid] {
        self.grid.inner.query_bucket(pos)
    }
}
