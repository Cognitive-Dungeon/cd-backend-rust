use crate::runtime::engine::Engine;
use crate::world::resources::MapResource;
use cd_ecs::InstanceId;
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
    pub fn load_chunk(
        &mut self,
        instance: InstanceId,
        chunk_x: i32,
        chunk_y: i32,
        chunk: cd_map::Chunk,
    ) {
        let chunk_key = cd_core::WorldPos::new(chunk_x, chunk_y, 0);
        let mut map_res = self.world.get_resource_mut::<MapResource>().unwrap();

        // Получаем карту конкретного инстанса и кладем туда чанк
        let map = map_res.get_mut_map(instance);
        map.put_chunk(chunk_key, chunk);
    }

    /// Загрузить чанк из репозитория.
    /// Если репозитория нет или чанка нет — тихо пропускает.
    pub fn load_chunk_from_repo(&mut self, instance: InstanceId, chunk_x: i32, chunk_y: i32) {
        let Some(ref repo) = self.world_repo else {
            return;
        };
        let key = cd_core::WorldPos::new(chunk_x, chunk_y, 0);

        // TODO: [Instancing] Репозиторий миров должен знать об инстансах.
        // Сейчас json репозиторий сохраняет/читает только по координатам X,Y,Z.
        // В будущем нужно расширить WorldRepository API до load_chunk(instance, key).
        match repo.load_chunk(key) {
            Ok(Some(chunk)) => {
                let mut map_res = self.world.get_resource_mut::<MapResource>().unwrap();
                let map = map_res.get_mut_map(instance);
                map.put_chunk(key, chunk);

                tracing::info!(
                    "Loaded chunk ({}, {}) from repository into instance {:?}",
                    chunk_x,
                    chunk_y,
                    instance
                );
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
