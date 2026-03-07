use crate::game_world::GameWorld;
use crate::registry::EntityRegistry;
use crate::system_runner::SystemRunner;
use crate::{EntitySnapshot, input::InputCmd};
use crate::{StampedCommand, TickContext, TickId, systems};
use cd_core::{ObjectGuid, WorldPos};
use cd_data::{EntityRepository, WorldRepository, depot::Depot};
use cd_ecs::components::{Name, Position, Render, Stats};
use cd_map::{SpatialGrid, WorldMap};
use cd_telemetry::{EngineEvent, NullSink, TelemetrySink};
use hecs::{CommandBuffer, World};
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

pub struct Engine {
    // ECS
    world: World,

    // Инфраструктура
    map: WorldMap,
    grid: SpatialGrid,

    // Буфер структурных изменений (Spawn/Despawn)
    cmd_buffer: CommandBuffer,
    entity_registry: EntityRegistry,

    /// Seed для воспроизводимого RNG (задаётся при создании)
    world_seed: u64,
    /// Текущий тик (монотонно растёт)
    current_tick: TickId,

    telemetry: Arc<dyn TelemetrySink>,
    world_repo: Option<Arc<dyn WorldRepository>>,
    entity_repo: Option<Arc<dyn EntityRepository>>,
    system_runner: SystemRunner,
    /// Игровые данные из Depot. RwLock — движок пишет при reload, системы читают.
    game_data: Arc<RwLock<Option<Depot>>>,
}

impl Engine {
    /// Создаётся только через EngineBuilder — явные зависимости.
    pub(crate) fn from_builder(
        world_seed: u64,
        telemetry: Arc<dyn TelemetrySink>,
        world_repo: Option<Arc<dyn WorldRepository>>,
        entity_repo: Option<Arc<dyn EntityRepository>>,
    ) -> Self {
        Self {
            world: World::new(),
            map: WorldMap::new(),
            grid: SpatialGrid::new(),
            cmd_buffer: CommandBuffer::new(),
            entity_registry: EntityRegistry::new(),
            world_seed,
            current_tick: TickId::default(),
            telemetry,
            world_repo,
            entity_repo,
            system_runner: SystemRunner::new(),
            game_data: Arc::new(RwLock::new(None)),
        }
    }

    /// Корректное завершение: сохраняем всё что можно перед выходом.
    /// Вызывается после последнего тика, до уничтожения Engine.
    pub fn shutdown(&mut self) {
        tracing::info!("Engine shutting down at {}", self.current_tick);

        self.flush_dirty_chunks();

        self.telemetry.emit(EngineEvent::ErrorIsolated {
            tick_id: self.current_tick.0,
            context: "engine".to_string(),
            error: "clean shutdown".to_string(),
        });

        tracing::info!("Engine shutdown complete");
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
        let chunk_key = cd_core::WorldPos::new(0, 0, 0);
        let tile = self.map.get_tile(chunk_key);

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
                        tick_id: self.current_tick.0,
                        context: "depot_reload".to_string(),
                        error: format!("Depot reloaded from {:?}", path),
                    });
            }
            Err(e) => {
                tracing::error!("Failed to reload depot: {}", e);
                self.telemetry
                    .emit(cd_telemetry::EngineEvent::ErrorIsolated {
                        tick_id: self.current_tick.0,
                        context: "depot_reload".to_string(),
                        error: e.to_string(),
                    });
            }
        }
    }

    /// Зарегистрировать систему. Системы выполняются в порядке регистрации.
    pub fn register_system(
        &mut self,
        name: &'static str,
        f: impl Fn(&mut GameWorld, &mut TickContext) -> Result<(), crate::game_error::GameError>
        + Send
        + 'static,
    ) {
        self.system_runner.register(name, f);
    }

    /// Создание сущности (Фабрика)
    pub fn spawn_player(&mut self, guid: ObjectGuid, name: String, pos: WorldPos) {
        // 1. Создаем в ECS
        let entity = self.world.spawn((
            Position(pos),
            Name(name.clone()),
            Render {
                glyph: '@',
                color_rgb: 0x00FF00,
            },
            Stats {
                hp: 100,
                max_hp: 100,
                mana: 100,
                max_mana: 100,
            },
            // Важно: храним GUID внутри компонента тоже, для обратного поиска
            cd_ecs::components::Controller {
                agent_id: "player".into(),
            },
        ));

        // 2. Регистрируем в регистрах
        self.entity_registry.register(guid, entity);
        self.grid.insert(guid, pos);

        info!("Spawned [{}] {} at {:?}", guid, name, pos);
        self.telemetry.emit(EngineEvent::EntitySpawned {
            tick_id: self.current_tick.0,
            guid: guid.to_string(),
            x: pos.x(),
            y: pos.y(),
        });
    }

    /// Главный цикл симуляции (Tick)
    pub fn tick(&mut self, commands: Vec<StampedCommand>) {
        let start = std::time::Instant::now();
        let tick_id = self.current_tick.0;
        let command_count = commands.len() as u32;

        let mut ctx = TickContext::new(self.world_seed, self.current_tick);

        // Входящие команды
        for stamped in commands {
            self.handle_input(stamped.payload);
        }

        // Берём runner из self чтобы обойти borrow checker
        // (runner нужен &mut self.world, &mut self.map и т.д. одновременно)
        let mut runner: SystemRunner = std::mem::take(&mut self.system_runner);

        {
            let mut gw = GameWorld {
                world: &mut self.world,
                map: &mut self.map,
                grid: &mut self.grid,
                registry: &mut self.entity_registry,
                commands: &mut self.cmd_buffer,
                telemetry: self.telemetry.as_ref(),
                game_data: Arc::clone(&self.game_data),
            };
            runner.run(&mut gw, &mut ctx, self.telemetry.as_ref());
        }

        self.system_runner = runner;

        // Применяем отложенные spawn/despawn
        self.cmd_buffer.run_on(&mut self.world);

        self.telemetry.emit(EngineEvent::TickCompleted {
            tick_id,
            duration_us: start.elapsed().as_micros() as u64,
            entity_count: self.world.len() as u32,
            command_count,
        });

        self.current_tick = self.current_tick.next();
    }

    fn handle_input(&mut self, cmd: InputCmd) {
        match cmd {
            InputCmd::SpawnPlayer { entity_guid, name } => {
                let spawn_pos = WorldPos::new(0, 0, 0);
                self.spawn_player(entity_guid, name, spawn_pos);
            }
            InputCmd::Move {
                entity_guid,
                target,
            } => {
                // Находим hecs::Entity по GUID
                if let Some(entity) = self.entity_registry.get_entity(entity_guid) {
                    // Добавляем/Обновляем компонент "TargetPosition" или просто телепортируем пока для теста
                    // В реальной игре тут мы бы добавили компонент IntentMove

                    // ХАК для теста: просто меняем позицию, если нет стены
                    // В нормальной системе это сделает movement_system
                    if !self.map.is_solid_fast(target) {
                        // Получаем доступ к позиции
                        if let Ok(mut pos) = self.world.get::<&mut Position>(entity) {
                            let old_pos = pos.0;
                            pos.0 = target;
                            // Обновляем Grid
                            self.grid.move_entity(entity_guid, old_pos, target);
                            info!("Entity {} moved to {:?}", entity_guid, target);
                        }
                    } else {
                        warn!("Entity {} hit a wall at {:?}", entity_guid, target);
                    }
                } else {
                    warn!("Input for unknown entity: {:?}", entity_guid);
                }
            }
            _ => {} // Пока игнорируем остальное
        }
    }

    /// Публичный снапшот для сетевого слоя.
    pub fn snapshot_entities(&self) -> Vec<EntitySnapshot> {
        self.world
            .query::<(&Position, &Render)>()
            .iter()
            .map(|(entity, (pos, render))| EntitySnapshot {
                guid: self.entity_registry.get_guid(entity),
                x: pos.0.x(),
                y: pos.0.y(),
                glyph: render.glyph,
                color_rgb: render.color_rgb,
            })
            .collect()
    }

    /// Загрузить чанк напрямую (например, при генерации мира).
    pub fn load_chunk(&mut self, chunk_x: i32, chunk_y: i32, chunk: cd_map::Chunk) {
        let chunk_key = cd_core::WorldPos::new(chunk_x, chunk_y, 0);
        self.map.put_chunk(chunk_key, chunk);
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
                self.map.put_chunk(key, chunk);
                tracing::info!("Loaded chunk ({}, {}) from repository", chunk_x, chunk_y);
            }
            Ok(None) => {
                tracing::debug!("Chunk ({}, {}) not found in repository", chunk_x, chunk_y);
            }
            Err(e) => {
                tracing::warn!("Failed to load chunk ({}, {}): {}", chunk_x, chunk_y, e);
                self.telemetry.emit(EngineEvent::ErrorIsolated {
                    tick_id: self.current_tick.0,
                    context: format!("load_chunk_from_repo ({}, {})", chunk_x, chunk_y),
                    error: e.to_string(),
                });
            }
        }
    }

    pub fn current_tick(&self) -> TickId {
        self.current_tick
    }
}
