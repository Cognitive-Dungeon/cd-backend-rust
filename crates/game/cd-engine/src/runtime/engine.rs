use crate::StampedCommand;
use crate::input::InputCmd;
use crate::systems::intents::IntentMove;
use crate::world::resources::*;
use bevy_ecs::message::Messages;
use bevy_ecs::prelude::*;
use cd_core::{ObjectGuid, WorldPos};
use cd_data::depot::Depot;
use cd_ecs::components::{Name, Position, Render, Stats};
use cd_ecs::{Controller, Guid, SpatialGrid};
use cd_map::WorldMap;
use cd_telemetry::{EngineEvent, TelemetrySink};
use std::sync::{Arc, RwLock};
use tracing::info;

pub struct Engine {
    // Вся память игры здесь (Сущности + Ресурсы)
    pub world: World,
    // Планировщик, который знает, в каком порядке запускать системы
    pub schedule: Schedule,
    // Сохраняем handle на Depot, чтобы файл-вотчер мог его обновлять
    game_data: Arc<RwLock<Option<Depot>>>,
    telemetry: Arc<dyn TelemetrySink>,
    world_repo: Option<Arc<dyn cd_data::WorldRepository>>,
    entity_repo: Option<Arc<dyn cd_data::EntityRepository>>,
}

impl Engine {
    /// Создаётся только через EngineBuilder — явные зависимости.
    pub(crate) fn from_builder(
        world_seed: u64,
        telemetry: Arc<dyn TelemetrySink>,
        world_repo: Option<Arc<dyn cd_data::WorldRepository>>, // Пока игнорируем для простоты
        entity_repo: Option<Arc<dyn cd_data::EntityRepository>>,
        game_data: Arc<RwLock<Option<Depot>>>,
    ) -> Self {
        let mut world = World::new();
        world.insert_resource(MapResource {
            inner: WorldMap::new(),
        });
        world.insert_resource(GridResource {
            inner: SpatialGrid::new(),
        });
        world.insert_resource(GameDataResource {
            depot: Arc::clone(&game_data),
        });
        world.insert_resource(RegistryResource::default());
        world.insert_resource(TickResource {
            id: crate::TickId(0),
            world_seed,
        });
        world.insert_resource(TelemetryResource(Arc::clone(&telemetry)));

        world.init_resource::<Messages<InputCmd>>();
        world.init_resource::<Messages<IntentMove>>();
        world.init_resource::<DefsCache>();

        Self {
            world,
            schedule: Schedule::default(),
            telemetry,
            game_data,
            world_repo,
            entity_repo,
        }
    }

    /// Корректное завершение: сохраняем всё что можно перед выходом.
    /// Вызывается после последнего тика, до уничтожения Engine.
    pub fn shutdown(&mut self) {
        tracing::info!("Engine shutting down at {}", self.current_tick().0);

        self.flush_dirty_chunks();

        self.telemetry.emit(EngineEvent::ErrorIsolated {
            tick_id: self.current_tick().0,
            context: "engine".to_string(),
            error: "clean shutdown".to_string(),
        });

        tracing::info!("Engine shutdown complete");
    }

    pub fn add_system<Params>(&mut self, system: impl IntoSystem<(), (), Params>) {
        self.schedule.add_systems(system);
    }

    /// Сбрасывает изменённые чанки в репозиторий.
    /// Вызывается при shutdown и опционально каждые N тиков.
    fn flush_dirty_chunks(&mut self) {
        let Some(ref repo) = self.world_repo else {
            tracing::debug!("No world_repo configured, skipping chunk flush");
            return;
        };

        // Собираем чанки которые нужно сохранить
        // Пока сохраняем тестовый чанк (0,0) — в будущем WorldMap будет
        // отслеживать dirty-флаги через DirtyTracker
        let map = self
            .world
            .get_resource::<MapResource>()
            .expect("MapRes must exist");
        let chunk_key = cd_core::WorldPos::new(0, 0, 0);
        let tile = map.inner.get_tile(chunk_key);

        // Временно: строим чанк из текущего состояния карты для сохранения
        // TODO: WorldMap::iter_dirty_chunks() когда добавим dirty tracking
        tracing::debug!("Chunk flush placeholder — dirty tracking not yet implemented");
    }

    /// Клонируемый handle на Depot — для передачи в file watcher и API.
    pub fn game_data_handle(&self) -> Arc<RwLock<Option<Depot>>> {
        Arc::clone(&self.game_data)
    }

    /// Перезагрузить данные из файла (вызывается file watcher'ом или VS Code).
    pub fn reload_game_data(&self, path: &std::path::Path) {
        match Depot::load(path) {
            Ok(depot) => {
                *self.game_data.write().unwrap() = Some(depot);
                tracing::info!("Depot reloaded from {:?}", path);
                self.telemetry
                    .emit(cd_telemetry::EngineEvent::ErrorIsolated {
                        tick_id: self.current_tick().0,
                        context: "depot_reload".to_string(),
                        error: format!("Depot reloaded from {:?}", path),
                    });
            }
            Err(e) => {
                tracing::error!("Failed to reload depot: {}", e);
                self.telemetry
                    .emit(cd_telemetry::EngineEvent::ErrorIsolated {
                        tick_id: self.current_tick().0,
                        context: "depot_reload".to_string(),
                        error: e.to_string(),
                    });
            }
        }
    }

    /// Принудительно заставляет ECS перечитать данные из Depot в DefsCache
    pub fn rebuild_cache(&mut self) {
        self.world
            .resource_scope(|world, mut cache: Mut<DefsCache>| {
                let game_data_res = world.get_resource::<GameDataResource>().unwrap();
                let guard = game_data_res.depot.read().unwrap();

                if let Some(depot) = guard.as_ref() {
                    cache.rebuild_from(depot);
                } else {
                    tracing::warn!("rebuild_cache called, but Depot is empty!");
                }
            });
    }

    /// Создание сущности (Фабрика)
    pub fn spawn_player(&mut self, guid: ObjectGuid, name: String, pos: WorldPos) {
        // 1. Создаем в ECS
        let entity = self
            .world
            .spawn((
                Guid(guid),
                Position(pos),
                Name(name.clone()),
                Render {
                    glyph: cd_common::Glyph::new(0x00FF00, b'@'),
                },
                Stats {
                    hp: 100,
                    max_hp: 100,
                    mana: 100,
                    max_mana: 100,
                },
                Controller {
                    agent_id: "player".into(),
                },
            ))
            .id();

        if let Some(mut registry) = self.world.get_resource_mut::<RegistryResource>() {
            registry.inner.register(guid, entity);
        }

        if let Some(mut grid) = self.world.get_resource_mut::<GridResource>() {
            grid.inner.insert(guid, pos);
        }

        info!("Spawned [{}] {} at {:?}", guid, name, pos);
        self.telemetry.emit(EngineEvent::EntitySpawned {
            tick_id: self.current_tick().0,
            guid: guid.to_string(),
            x: pos.x(),
            y: pos.y(),
        });
    }

    /// Главный цикл симуляции (Tick)
    pub fn tick(&mut self, commands: Vec<StampedCommand>) {
        let start = std::time::Instant::now();
        let command_count = commands.len() as u32;

        let tick_id = {
            let mut tick = self.world.get_resource_mut::<TickResource>().unwrap();
            tick.id = tick.id.next();
            tick.id.0
        };

        // 2. Пишем команды из сети в ECS-ресурс Messages
        if let Some(mut messages) = self.world.get_resource_mut::<Messages<InputCmd>>() {
            for stamped in commands {
                messages.write(stamped.payload);
            }
        }

        // Запуск всех систем
        self.schedule.run(&mut self.world);

        self.world.clear_trackers();

        // Возвращаем событие TickCompleted
        self.telemetry.emit(EngineEvent::TickCompleted {
            tick_id,
            duration_us: start.elapsed().as_micros() as u64,
            entity_count: self.world.entities().len(), // Bevy умеет быстро считать сущности
            command_count,
        });
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
                let mut map = self.world.get_resource_mut::<MapResource>().unwrap();
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

    pub fn current_tick(&self) -> crate::TickId {
        self.world
            .get_resource::<TickResource>()
            .map(|t| t.id)
            .unwrap_or(crate::TickId(0))
    }
}
