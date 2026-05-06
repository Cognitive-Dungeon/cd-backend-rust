use bevy_ecs::prelude::*;
use cd_core::WorldPos;
use cd_ecs::{
    Guid,
    components::{Controller, Name, Position, Render, Stats},
};
use cd_telemetry::EngineEvent;

use crate::{
    input::InputCmd,
    systems::intents::{IntentCastSpell, IntentMove},
    world::{
        factory::EntityFactoryExt as _,
        resources::{DefsCache, GridResource, RegistryResource, TelemetryResource, TickResource},
    },
};

pub fn handle_input_system(
    mut reader: MessageReader<InputCmd>, // <--- Читаем буфер сообщений
    mut commands: Commands,              // <--- Для спавна новых сущностей
    mut intent_move_writer: MessageWriter<IntentMove>,
    mut registry: ResMut<RegistryResource>,
    defs: Res<DefsCache>,
    mut grid: ResMut<GridResource>,
    telemetry: Res<TelemetryResource>,
    tick: Res<TickResource>,
    mut positions: Query<&mut Position>, // <--- Запрашиваем только позиции
    mut intent_cast_writer: MessageWriter<IntentCastSpell>,
) {
    // Разгребаем все сообщения, накопившиеся за тик
    for cmd in reader.read() {
        match cmd {
            InputCmd::SpawnPlayer { entity_guid, name } => {
                let pos = WorldPos::new(3, 6, 0);

                if let Some(entity) =
                    commands.spawn_creature("human", *entity_guid, pos, name.clone(), &defs, true)
                {
                    // Обновляем индексы
                    registry.inner.register(*entity_guid, entity);
                    grid.inner.insert(*entity_guid, pos);

                    telemetry.0.emit(EngineEvent::EntitySpawned {
                        tick_id: tick.id.0,
                        guid: entity_guid.to_string(),
                        x: pos.x(),
                        y: pos.y(),
                    });
                    tracing::info!("Spawned [{}] {} at {:?}", entity_guid, name, pos);
                }
            }

            InputCmd::Move {
                entity_guid,
                target,
            } => {
                if let Some(entity) = registry.inner.get_entity(*entity_guid) {
                    intent_move_writer.write(IntentMove {
                        entity,
                        target: *target,
                    });
                }
            }

            InputCmd::CastSpell {
                entity_guid,
                spell_slug,
            } => {
                // Резолвим slug → SpellId через кэш
                let Some(&spell_id) = defs.slug_to_spell.get(spell_slug) else {
                    tracing::warn!("Unknown spell slug: {}", spell_slug);
                    continue;
                };

                if let Some(entity) = registry.inner.get_entity(*entity_guid) {
                    intent_cast_writer.write(IntentCastSpell {
                        caster: entity,
                        spell_id,
                    });
                }
            }

            _ => {}
        }
    }
}
