use crate::registry::EntityRegistry;
use crate::systems;
use crate::{EntitySnapshot, input::InputCmd};
use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::components::{Name, Position, Render, Stats};
use cd_map::{SpatialGrid, WorldMap};
use hecs::{CommandBuffer, World};
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
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            world: World::new(),
            map: WorldMap::new(),
            grid: SpatialGrid::new(),
            cmd_buffer: CommandBuffer::new(),
            entity_registry: EntityRegistry::new(),
        }
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
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
    }

    /// Главный цикл симуляции (Tick)
    pub fn tick(&mut self, inputs: Vec<InputCmd>) {
        // 1. Process Input (Cmd -> Component State/Intent)
        for cmd in inputs {
            self.handle_input(cmd);
        }

        // 2. Logic Systems
        // Передаем &mut self.world, чтобы системы могли итерироваться
        // Но для сложных систем нам понадобится Context, пока сделаем просто функцию
        systems::movement::run_movement(&mut self.world, &self.map, &mut self.grid);

        // 3. Apply Structural Changes (если системы просили удалить/создать сущности)
        self.cmd_buffer.run_on(&mut self.world);
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
}
