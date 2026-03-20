use bevy_ecs::prelude::*;
use cd_core::WorldPos;
use cd_ecs::{
    Guid,
    components::{Controller, Name, Position, Render, Stats},
};
use cd_telemetry::EngineEvent;

use crate::{
    input::InputCmd,
    systems::intents::IntentMove,
    world::resources::{
        DefsCache, GridResource, RegistryResource, TelemetryResource, TickResource,
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
) {
    // Разгребаем все сообщения, накопившиеся за тик
    for cmd in reader.read() {
        match cmd {
            InputCmd::SpawnPlayer { entity_guid, name } => {
                let pos = WorldPos::new(0, 0, 0);

                let creature_id = "human"; // В будущем это будет браться из команды
                let Some(def) = defs.creatures.get(creature_id) else {
                    tracing::error!("Creature '{}' not found in Depot!", creature_id);
                    continue; // Пропускаем спавн
                };

                // Спавним через Commands
                let entity = commands
                    .spawn((
                        Guid(*entity_guid),
                        Position(pos),
                        Name(name.clone()),
                        Render { glyph: def.glyph },
                        Stats {
                            hp: def.base_hp,
                            max_hp: def.base_hp,
                            mana: def.base_mp,
                            max_mana: def.base_mp,
                        },
                        Controller {
                            agent_id: "player".into(),
                        },
                    ))
                    .id();

                // Обновляем ресурсы
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

            _ => {}
        }
    }
}
