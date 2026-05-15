use crate::input::InputCmd;
use crate::systems::intents::{IntentEndTurn, IntentMove};
use crate::world::resources::*;
use crate::{StampedCommand, systems::intents::IntentCastSpell};
use bevy::ecs::message::Messages;
use bevy::ecs::prelude::*;
use cd_data::provider::DataProvider;
use cd_ecs::SpatialGrid;
use cd_telemetry::{EngineEvent, TelemetrySink};
use std::sync::Arc;

pub struct Engine {
    // Вся память игры здесь (Сущности + Ресурсы)
    pub world: World,
    // Планировщик, который знает, в каком порядке запускать системы
    pub schedule: Schedule,
    pub(crate) telemetry: Arc<dyn TelemetrySink>,
    pub(crate) world_repo: Option<Arc<dyn cd_data::WorldRepository>>,
    entity_repo: Option<Arc<dyn cd_data::EntityRepository>>,
}

impl Engine {
    /// Создаётся только через EngineBuilder — явные зависимости.
    pub(crate) fn from_builder(
        world_seed: u64,
        telemetry: Arc<dyn TelemetrySink>,
        world_repo: Option<Arc<dyn cd_data::WorldRepository>>,
        entity_repo: Option<Arc<dyn cd_data::EntityRepository>>,
        data_provider: Arc<dyn DataProvider>,
    ) -> Self {
        let mut world = World::new();
        world.init_resource::<MapResource>();
        world.insert_resource(GridResource {
            inner: SpatialGrid::new(),
        });
        world.insert_resource(GameDataResource {
            provider: data_provider,
        });
        world.insert_resource(RegistryResource::default());
        world.insert_resource(TickResource {
            id: crate::TickId(0),
            world_seed,
        });
        world.insert_resource(TelemetryResource(Arc::clone(&telemetry)));

        world.init_resource::<Messages<InputCmd>>();
        world.init_resource::<Messages<IntentMove>>();
        world.init_resource::<Messages<IntentCastSpell>>();
        world.init_resource::<Messages<IntentEndTurn>>();
        world.init_resource::<DefsCache>();

        Self {
            world,
            schedule: Schedule::default(),
            telemetry,
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

    /// Принудительно заставляет ECS перечитать данные через DataProvider в DefsCache
    pub fn rebuild_cache(&mut self) {
        self.world
            .resource_scope(|world, mut cache: Mut<DefsCache>| {
                let game_data_res = world
                    .get_resource::<GameDataResource>()
                    .expect("GameDataResource must be present!");

                // Провайдер сам сходит на диск/в БД и обновит кэш.
                cache.rebuild_from(game_data_res.provider.as_ref());
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

    pub fn current_tick(&self) -> crate::TickId {
        self.world
            .get_resource::<TickResource>()
            .map(|t| t.id)
            .unwrap_or(crate::TickId(0))
    }
}
