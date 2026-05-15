use anyhow::Result;
use bevy::app::App;
use cd_data::json::{JsonEntityRepository, JsonWorldRepository};
use cd_data::provider::RonDataProvider;
use cd_ecs::InstanceId;
use cd_engine::{BroadcastSink, CommandBus, InputCmd, runtime::plugin::EnginePlugin};
use cd_net::protocol::{OutboundMessage, ServerPacket, TileView};
use cd_net::snapshot::SnapshotBuilder;
use cd_net::{ApiEntity, ApiState, ReloadCallback, SharedApiState};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, oneshot};
use tracing::info;

pub fn run() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::new("info,wgpu=warn,wgpu_hal=error,naga=warn");
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("🚀 Booting Cognitive Dungeon...");

    // 1. Инициализируем Tokio вручную, чтобы он ушел в фон
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let data_path = "./data".to_string();
    let (cmd_bus, cmd_sender) = CommandBus::new(1024);
    let (outbound_tx, _) = broadcast::channel::<OutboundMessage>(16);
    let (telemetry_sink, telemetry_tx) = BroadcastSink::new(256);
    let (net_stop_tx, net_stop_rx) = oneshot::channel();
    let api_state: SharedApiState = Arc::new(Mutex::new(ApiState::default()));

    let data_provider = Arc::new(RonDataProvider::new(&data_path));
    let world_repo = Arc::new(JsonWorldRepository::new(&data_path)?);
    let entity_repo = Arc::new(JsonEntityRepository::new(&data_path)?);

    let reload_cb: ReloadCallback = {
        let cmd_tx = cmd_sender.clone();
        Arc::new(tokio::sync::Mutex::new(Box::new(move || {
            let sender = cmd_tx.clone();
            tokio::spawn(async move {
                let _ = sender.send(InputCmd::ReloadData).await;
            });
        })))
    };

    // 2. ЗАПУСКАЕМ СЕТЬ В ФОНЕ (Tokio)
    tokio::spawn(cd_net::run_server(
        8080,
        cmd_sender,
        outbound_tx.subscribe(),
        telemetry_tx,
        net_stop_rx,
        api_state.clone(),
        reload_cb,
    ));

    // 3. СОБИРАЕМ BEVY APP
    let mut app = App::new();

    // -- РЕЖИМ РАЗРАБОТКИ (Включается через фичу) --
    #[cfg(feature = "dev_editor")]
    {
        use bevy::app::PluginGroup;

        app.add_plugins(
            bevy::DefaultPlugins
                .build()
                .disable::<bevy::log::LogPlugin>()
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "CD Engine - God Mode".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        );
        app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
        app.add_plugins(bevy_inspector_egui::quick::FilterQueryInspectorPlugin::<
            bevy::ecs::query::With<cd_ecs::Guid>,
        >::default());

        cd_core::editor::register_core_editor_uis(&mut app);
    }

    // -- ПРОДАКШЕН РЕЖИМ (Без графики, 20 TPS) --
    #[cfg(not(feature = "dev_editor"))]
    {
        use std::time::Duration;
        app.add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(
            Duration::from_millis(50),
        ));
    }

    // Подключаем наш движок
    app.add_plugins(EnginePlugin {
        data_provider,
        world_repo,
        entity_repo,
        telemetry: Arc::new(telemetry_sink),
        world_seed: 0xDEAD_CAFE_BABE_1337,
    });

    // Прокидываем каналы связи внутрь ECS
    app.insert_non_send_resource(cmd_bus);
    app.insert_non_send_resource(outbound_tx);
    app.insert_non_send_resource(api_state);

    // Добавляем системы-мосты
    app.add_systems(bevy::app::PreUpdate, receive_network_commands);
    app.add_systems(bevy::app::PostUpdate, publish_state_system);
    app.add_systems(bevy::app::Startup, setup_test_world); // Спавн тестовых мобов

    // 4. ЗАПУСК ДВИЖКА (Блокирует главный поток)
    info!("✅ Engine initialized. Taking over main thread.");
    app.run();

    Ok(())
}

// ============================================================================
// Системы-мосты (Связь Сети и Движка)
// ============================================================================

fn receive_network_commands(
    mut cmd_bus: bevy::ecs::system::NonSendMut<CommandBus>,
    mut messages: bevy::ecs::message::MessageWriter<InputCmd>,
) {
    for cmd in cmd_bus.drain_sorted() {
        messages.write(cmd.payload);
    }
}

/// Эксклюзивная система (имеет доступ ко всему World сразу)
fn publish_state_system(world: &mut bevy::ecs::world::World) {
    let tick = world
        .resource::<cd_engine::world::resources::TickResource>()
        .id
        .0;

    // Продвигаем тик (так как у нас больше нет внешнего цикла)
    world
        .resource_mut::<cd_engine::world::resources::TickResource>()
        .id
        .0 += 1;

    // TODO: [Instancing] Сейчас мы жестко отдаем клиентам только OVERWORLD.
    // В будущем нужно собирать разные снапшоты для клиентов на разных этажах!
    let target_instance = cd_ecs::InstanceId::OVERWORLD;

    let snapshots = cd_net::snapshot::SnapshotBuilder::build_entities(world, target_instance);
    let api_state = world.non_send_resource::<SharedApiState>().clone();
    let outbound_tx = world
        .non_send_resource::<broadcast::Sender<OutboundMessage>>()
        .clone();

    if let Ok(mut state) = api_state.lock() {
        state.tick = tick;
        state.entity_count = snapshots.len() as u32;
        state.entities = snapshots
            .iter()
            .map(|s| ApiEntity {
                guid: s.guid.map(|g| g.to_string()).unwrap_or_default(),
                x: s.x,
                y: s.y,
                glyph: s.glyph.to_char(),
                color: s.glyph.hex_color(),
            })
            .collect();
    }

    let entities_view: Vec<cd_net::protocol::EntityView> = snapshots
        .into_iter()
        .map(|snap| cd_net::protocol::EntityView {
            guid: snap.guid.map(|g| g.to_string()).unwrap_or_default(),
            x: snap.x,
            y: snap.y,
            glyph: snap.glyph.to_char(),
            color: snap.glyph.hex_color(),
            hp: snap.hp,
            max_hp: snap.max_hp,
        })
        .collect();

    let _ = outbound_tx.send(OutboundMessage::broadcast(ServerPacket::Snapshot {
        tick,
        entities: entities_view,
    }));

    if tick.is_multiple_of(20) {
        let chunk_pos = cd_core::WorldPos::new(0, 0, 0);
        let chunk_snap = SnapshotBuilder::build_chunk(world, target_instance, chunk_pos);
        let palette_view: Vec<TileView> = chunk_snap
            .palette
            .into_iter()
            .map(|g| TileView {
                glyph: g.to_char(),
                color: g.hex_color(),
            })
            .collect();
        let _ = outbound_tx.send(OutboundMessage::broadcast(ServerPacket::MapChunk {
            x: chunk_snap.chunk_x,
            y: chunk_snap.chunk_y,
            palette: palette_view,
            indices: chunk_snap.indices,
        }));
    }
}

fn setup_test_world(
    mut commands: bevy::ecs::system::Commands,
    mut map: bevy::ecs::system::ResMut<cd_engine::world::resources::MapResource>,
    defs: bevy::ecs::system::Res<cd_engine::world::resources::DefsCache>,
) {
    #[cfg(feature = "dev_editor")]
    commands.spawn(bevy::prelude::Camera2d);

    cd_engine::world::generator::WorldGenerator::generate_test_room(&mut map, &defs);

    // Спавним Гоблина через фабрику
    use cd_engine::world::factory::EntityFactoryExt;
    commands.spawn_creature(
        "skeleton",
        cd_core::ObjectGuid::new(1, 2, 1, 9999),
        cd_core::WorldPos::new(3, 3, 0),
        "Test Goblin",
        &defs,
        false,
        InstanceId::OVERWORLD,
    );
}
