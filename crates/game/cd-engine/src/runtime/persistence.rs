use crate::runtime::engine::Engine;
use crate::world::resources::MapResource;
use cd_telemetry::EngineEvent;

impl Engine {
    /// Сбрасывает изменённые чанки в репозиторий.
    /// Вызывается при shutdown и опционально каждые N тиков.
    pub(crate) fn flush_dirty_chunks(&mut self) {
        let Some(ref repo) = self.world_repo else {
            tracing::debug!("No world_repo configured, skipping chunk flush");
            return;
        };

        // TODO: WorldMap::iter_dirty_chunks() когда добавим dirty tracking
        let _ = repo; // подавляем unused warning пока не реализовано
        tracing::debug!("Chunk flush placeholder — dirty tracking not yet implemented");
    }

    /// Загрузить чанк напрямую (например, при генерации мира).
    pub fn load_chunk(&mut self, chunk_x: i32, chunk_y: i32, chunk: cd_map::Chunk) {
        let chunk_key = cd_core::WorldPos::new(chunk_x, chunk_y, 0);
        let map = self.world.get_resource_mut::<MapResource>().unwrap();
        map.inner.put_chunk(chunk_key, chunk);
    }

    /// Загрузить чанк из репозитория.
    /// Если репозитория нет или чанка нет — тихо пропускает.
    pub fn load_chunk_from_repo(&mut self, chunk_x: i32, chunk_y: i32) {
        let Some(ref repo) = self.world_repo else {
            return;
        };
        let key = cd_core::WorldPos::new(chunk_x, chunk_y, 0);

        match repo.load_chunk(key) {
            Ok(Some(chunk)) => {
                let map = self.world.get_resource_mut::<MapResource>().unwrap();
                map.inner.put_chunk(key, chunk);
                tracing::info!("Loaded chunk ({}, {}) from repository", chunk_x, chunk_y);
            }
            Ok(None) => {
                tracing::debug!("Chunk ({}, {}) not found in repository", chunk_x, chunk_y);
            }
            Err(e) => {
                tracing::warn!("Failed to load chunk ({}, {}): {}", chunk_x, chunk_y, e);
                self.telemetry.emit(EngineEvent::ErrorIsolated {
                    tick_id: self.current_tick().0,
                    context: format!("load_chunk_from_repo ({}, {})", chunk_x, chunk_y),
                    error: e.to_string(),
                });
            }
        }
    }
}
