use crate::input::InputCmd;
use crate::systems;
use cd_core::{ObjectGuid, WorldPos};
use cd_ecs::components::{Position, Name, Render, Stats};
use cd_map::{WorldMap, SpatialGrid};
use hecs::{World, Entity, CommandBuffer};
use std::collections::HashMap;
use tracing::{info, warn};
use crate::registry::EntityRegistry;

pub struct Engine {
    // ECS
    pub world: World,

    // Инфраструктура
    pub map: WorldMap,
    pub grid: SpatialGrid,

    // Маппинг GUID (наш ID) -> Entity (hecs ID)
    // Это критически важно для производительности O(1)
    entity_index: HashMap<ObjectGuid, Entity>,

    // Буфер структурных изменений (Spawn/Despawn)
    cmd_buffer: CommandBuffer,
    entity_registry: EntityRegistry
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            world: World::new(),
            map: WorldMap::new(),
            grid: SpatialGrid::new(),
            entity_index: HashMap::new(),
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
            Render { glyph: '@', color_rgb: 0x00FF00 },
            Stats { hp: 100, max_hp: 100, mana: 100, max_mana: 100 },
            // Важно: храним GUID внутри компонента тоже, для обратного поиска
            cd_ecs::components::Controller { agent_id: "player".into() },
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
        systems::movement::run_movement(&mut self.world, &self.map, &mut self.grid, &self.entity_index);

        // 3. Apply Structural Changes (если системы просили удалить/создать сущности)
        self.cmd_buffer.run_on(&mut self.world);
    }

    fn handle_input(&mut self, cmd: InputCmd) {
        match cmd {
            InputCmd::Move { entity_guid, target } => {
                // Находим hecs::Entity по GUID
                if let Some(&entity) = self.entity_index.get(&entity_guid) {
                    // Добавляем/Обновляем компонент "TargetPosition" или просто телепортируем пока для теста
                    // В реальной игре тут мы бы добавили компонент IntentMove

                    // ХАК для теста: просто меняем позицию, если нет стены
                    // В нормальной системе это сделает movement_system
                    if !self.map.is_solid(target) {
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
}