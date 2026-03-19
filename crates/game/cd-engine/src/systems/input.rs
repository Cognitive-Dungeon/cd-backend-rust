use bevy_ecs::prelude::*;
use cd_core::WorldPos;
use cd_ecs::{
    Guid,
    components::{Controller, Name, Position, Render, Stats},
};
use cd_telemetry::EngineEvent;

use crate::{
    input::InputCmd,
    world::resources::{
        GridResource, MapResource, RegistryResource, TelemetryResource, TickResource,
    },
};

pub fn handle_input_system(
    mut reader: MessageReader<InputCmd>, // <--- Читаем буфер сообщений
    mut commands: Commands,              // <--- Для спавна новых сущностей
    mut registry: ResMut<RegistryResource>,
    mut grid: ResMut<GridResource>,
    map: Res<MapResource>,
    telemetry: Res<TelemetryResource>,
    tick: Res<TickResource>,
    mut positions: Query<&mut Position>, // <--- Запрашиваем только позиции
) {
    // Разгребаем все сообщения, накопившиеся за тик
    for cmd in reader.read() {
        match cmd {
            InputCmd::SpawnPlayer { entity_guid, name } => {
                let pos = WorldPos::new(0, 0, 0);

                // Спавним через Commands
                let entity = commands
                    .spawn((
                        Guid(*entity_guid),
                        Position(pos),
                        Name(name.clone()),
                        Render {
                            glyph: cd_common::Glyph::new(0x00FF00, b'@'),
                        },
                        Stats {
                            hp: 100,
                            max_hp: 100,
                            mana: 100,
                            max_mana: 100,
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
                let entity_opt = registry.inner.get_entity(*entity_guid);

                if let Some(entity) = entity_opt {
                    if !map.inner.is_solid_fast(*target) {
                        // Пытаемся получить компонент Position у этой сущности
                        if let Ok(mut pos) = positions.get_mut(entity) {
                            let old_pos = pos.0;
                            pos.0 = *target;

                            grid.inner.move_entity(*entity_guid, old_pos, *target);
                            tracing::info!("Entity {} moved to {:?}", entity_guid, target);
                        }
                    } else {
                        tracing::warn!("Entity {} hit a wall at {:?}", entity_guid, target);
                    }
                }
            }

            _ => {}
        }
    }
}
