use bevy::app::{App, Plugin, Update};
use bevy::ecs::prelude::*;
use cd_data::{EntityRepository, WorldRepository, provider::DataProvider};
use cd_telemetry::TelemetrySink;
use std::sync::Arc;

use crate::{
    input::InputCmd,
    systems::{
        input::handle_input_system,
        movement::movement_system,
        spell::spell_system,
        turn::{combat_turn_system, npc_ai_system},
    },
    world::resources::{
        DefsCache, GameDataResource, GridResource, MapResource, RegistryResource,
        TelemetryResource, TickResource,
    },
};
use bevy::ecs::message::Messages;
use cd_ecs::SpatialGrid;

/// Главный плагин движка. Регистрирует все системы и ресурсы.
pub struct EnginePlugin {
    pub data_provider: Arc<dyn DataProvider>,
    pub world_repo: Arc<dyn WorldRepository>,
    pub entity_repo: Arc<dyn EntityRepository>,
    pub telemetry: Arc<dyn TelemetrySink>,
    pub world_seed: u64,
}

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        // 1. Вставляем ресурсы
        app.init_resource::<MapResource>();
        app.insert_resource(GridResource {
            inner: SpatialGrid::new(),
        });
        app.insert_resource(RegistryResource::default());
        app.insert_resource(TickResource {
            id: crate::TickId(0),
            world_seed: self.world_seed,
        });
        app.insert_resource(TelemetryResource(self.telemetry.clone()));
        app.insert_resource(GameDataResource {
            provider: self.data_provider.clone(),
        });

        // 2. Инициализируем кэш из файлов
        let mut cache = DefsCache::default();
        cache.rebuild_from(self.data_provider.as_ref());
        app.insert_resource(cache);

        // 3. Инициализируем буферы сообщений
        app.init_resource::<Messages<InputCmd>>();
        app.init_resource::<Messages<crate::systems::intents::IntentMove>>();
        app.init_resource::<Messages<crate::systems::intents::IntentCastSpell>>();
        app.init_resource::<Messages<crate::systems::intents::IntentEndTurn>>();

        // 4. Регистрируем наши системы в строгом порядке
        app.add_systems(
            Update,
            (
                handle_input_system,
                npc_ai_system,
                combat_turn_system,
                movement_system,
                spell_system,
            )
                .chain(),
        );
    }
}
