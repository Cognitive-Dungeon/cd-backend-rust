use bevy_ecs::prelude::*;
use cd_core::WorldPos;
use cd_ecs::Position;
use cd_telemetry::EngineEvent;

use crate::{
    input::InputCmd,
    systems::intents::{IntentCastSpell, IntentMove},
    world::{
        factory::EntityFactoryExt,
        resources::{DefsCache, TelemetryResource, TickResource},
        subsystems::SpatialSubsystem,
    },
};

pub fn handle_input_system(
    mut reader: MessageReader<InputCmd>,
    mut commands: Commands,
    mut intent_move_writer: MessageWriter<IntentMove>,
    mut spatial: SpatialSubsystem,
    defs: Res<DefsCache>,
    telemetry: Res<TelemetryResource>,
    tick: Res<TickResource>,
    mut intent_cast_writer: MessageWriter<IntentCastSpell>,
    positions: Query<&Position>,
) {
    for cmd in reader.read() {
        match cmd {
            InputCmd::SpawnPlayer { entity_guid, name } => {
                let pos = WorldPos::new(3, 6, 0);

                if let Some(entity) =
                    commands.spawn_creature("human", *entity_guid, pos, name.clone(), &defs, true)
                {
                    // Обновляем индексы в одну строку!
                    spatial.register_entity(*entity_guid, entity, pos);

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
                direction,
            } => {
                // Игнорируем пакеты "я никуда не иду"
                if *direction == cd_core::Direction::None {
                    continue;
                }

                if let Some(entity) = spatial.get_entity(*entity_guid) {
                    // Запрашиваем ТЕКУЩУЮ позицию игрока
                    if let Ok(pos) = positions.get(entity) {
                        // Получаем смещение (dx, dy, dz) из направления
                        let (dx, dy, dz) = direction.offset();

                        // Вычисляем целевую клетку
                        let target =
                            cd_core::WorldPos::new(pos.0.x() + dx, pos.0.y() + dy, pos.0.z() + dz);

                        tracing::info!(
                            "InputSystem: Processing Move {:?} for {} to {:?}",
                            direction,
                            entity_guid,
                            target
                        );

                        intent_move_writer.write(IntentMove { entity, target });
                    }
                }
            }

            InputCmd::CastSpell {
                entity_guid,
                spell_slug,
            } => {
                let Some(&spell_id) = defs.slug_to_spell.get(spell_slug) else {
                    tracing::warn!("Unknown spell slug: {}", spell_slug);
                    continue;
                };

                if let Some(entity) = spatial.get_entity(*entity_guid) {
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
