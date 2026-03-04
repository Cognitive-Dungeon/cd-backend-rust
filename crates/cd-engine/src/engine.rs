use crate::registry::EntityRegistry;
use crate::{EntitySnapshot, input::InputCmd};
use crate::{StampedCommand, TickContext, TickId, systems};
use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::components::{Name, Position, Render, Stats};
use cd_map::{SpatialGrid, WorldMap};
use cd_telemetry::{EngineEvent, NullSink, TelemetrySink};
use hecs::{CommandBuffer, World};
use std::sync::Arc;
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
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            world: World::new(),
            map: WorldMap::new(),
            grid: SpatialGrid::new(),
            cmd_buffer: CommandBuffer::new(),
            entity_registry: EntityRegistry::new(),
            world_seed: 0xDEAD_CAFE_BABE_1337,
            current_tick: TickId::default(),
            telemetry: Arc::new(NullSink),
        }
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_telemetry(sink: Arc<dyn TelemetrySink>) -> Self {
        Self {
            telemetry: sink,
            ..Self::default()
        }
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

        // Создаём детерминированный контекст для этого тика
        let _ctx = TickContext::new(self.world_seed, self.current_tick);

        for stamped in commands {
            self.handle_input(stamped.payload);
        }

        // 2. Logic Systems
        // Передаем &mut self.world, чтобы системы могли итерироваться
        // Но для сложных систем нам понадобится Context, пока сделаем просто функцию
        systems::movement::run_movement(&mut self.world, &self.map, &mut self.grid);

        // 3. Apply Structural Changes (если системы просили удалить/создать сущности)
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

    /// Загружает чанк в статический слой карты.
    /// Принимает chunk-координаты (не тайловые).
    pub fn load_chunk(&mut self, chunk_x: i32, chunk_y: i32, chunk: cd_map::Chunk) {
        let chunk_key = cd_core::WorldPos::new(chunk_x, chunk_y, 0);
        self.map.put_chunk(chunk_key, chunk);
    }

    pub fn current_tick(&self) -> TickId {
        self.current_tick
    }
}
