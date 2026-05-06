use bevy_ecs::prelude::*;
use cd_core::WorldPos;
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
                target,
            } => {
                tracing::info!(
                    "InputSystem: Processing Move for {} to {:?}",
                    entity_guid,
                    target
                );
                if let Some(entity) = spatial.get_entity(*entity_guid) {
                    intent_move_writer.write(IntentMove {
                        entity,
                        target: *target,
                    });
                    tracing::info!("InputSystem: IntentMove dispatched!");
                } else {
                    tracing::warn!(
                        "InputSystem: Entity {} not found in spatial registry!",
                        entity_guid
                    );
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
