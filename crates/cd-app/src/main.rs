// ПОЛНЫЙ ФАЙЛ:

use cd_core::{ObjectGuid, WorldPos};
use cd_data_json::{JsonEntityRepository, JsonWorldRepository};
use cd_engine::watcher::spawn_depot_watcher;
use cd_engine::{BroadcastSink, CommandBus, EngineBuilder};
use cd_map::{Chunk, Tile, TileFlags};
use cd_net::ReloadCallback;
use cd_net::protocol::{EntityView, ServerPacket};
use cd_net::{ApiEntity, ApiState, SharedApiState};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::broadcast;
use tracing::{Level, error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 Booting Cognitive Dungeon...");

    // --- Каналы ---
    let (mut cmd_bus, cmd_sender) = CommandBus::new(1024);
    let (snapshot_tx, _) = broadcast::channel::<ServerPacket>(16);
    let snapshot_tx_net = snapshot_tx.clone();

    let (telemetry_sink, telemetry_tx) = BroadcastSink::new(256);
    let telemetry_sink = Arc::new(telemetry_sink);

    // --- Репозитории ---
    let world_repo =
        Arc::new(JsonWorldRepository::new("./data").expect("Failed to init world repository"));
    let entity_repo =
        Arc::new(JsonEntityRepository::new("./data").expect("Failed to init entity repository"));

    let game_data: Arc<std::sync::RwLock<Option<cd_depot::Depot>>> =
        Arc::new(std::sync::RwLock::new(None));
    let game_data_engine  = game_data.clone(); // для треда
    let game_data_api     = game_data.clone(); // для reload callback

    let depot_path_api = std::path::PathBuf::from("./data/game.cdb");
    let reload_cb: cd_net::ReloadCallback = Arc::new(tokio::sync::Mutex::new(
        Box::new(move || {
            match cd_depot::Depot::load(&depot_path_api) {
                Ok(depot) => *game_data_api.write().unwrap() = Some(depot),
                Err(e)    => tracing::error!("API reload failed: {}", e),
            }
        })
    ));
    let reload_cb_net = reload_cb.clone();
    
    // --- Сигнал остановки ---
    // Используем пару каналов: main -> engine_thread и main -> net
    let (engine_stop_tx, engine_stop_rx) = tokio::sync::oneshot::channel::<()>();
    let (net_stop_tx, net_stop_rx) = tokio::sync::oneshot::channel::<()>();

    let api_state: SharedApiState = Arc::new(Mutex::new(ApiState::default()));
    let api_state_engine = api_state.clone();
    let api_state_net = api_state.clone();

    // --- Engine Thread (CPU-bound, отдельный OS поток) ---
    let engine_handle = std::thread::spawn(move || {
        let mut engine = EngineBuilder::new()
            .telemetry(telemetry_sink)
            .world_repo(world_repo)
            .entity_repo(entity_repo)
            .world_seed(0xDEAD_CAFE_BABE_1337)
            .build();

        // Путь к .cdb файлу (можно вынести в config)
        let depot_path = std::path::PathBuf::from("./data/game.cdb");
        if depot_path.exists() {
            match cd_depot::Depot::load(&depot_path) {
                Ok(depot) => *game_data_engine.write().unwrap() = Some(depot),
                Err(e)    => tracing::error!("Failed to load depot: {}", e),
            }
        }

        let game_data_watcher = game_data_engine.clone();
        let _watcher = cd_engine::watcher::spawn_depot_watcher(
            depot_path.clone(),
            move |path| {
                match cd_depot::Depot::load(path) {
                    Ok(depot) => *game_data_watcher.write().unwrap() = Some(depot),
                    Err(e)    => tracing::error!("Hot reload failed: {}", e),
                }
            },
        );

        engine.register_system("movement", cd_engine::systems::movement::run);

        // Setup начального состояния мира
        let mut chunk = Chunk::new();
        chunk.set_tile(
            5,
            5,
            Tile {
                material: 1,
                flags: TileFlags::SOLID,
                variant: 0,
            },
        );
        engine.load_chunk(0, 0, chunk);

        let player_guid = ObjectGuid::new(1, 1, 1, 4);
        engine.spawn_player(player_guid, "NetPlayer".to_string(), WorldPos::new(0, 0, 0));

        let tick_rate = Duration::from_millis(50); // 20 TPS
        let mut engine_stop_rx = engine_stop_rx;

        loop {
            let start = std::time::Instant::now();

            // Проверяем сигнал остановки (non-blocking)
            match engine_stop_rx.try_recv() {
                Ok(_) => {
                    info!("Engine received stop signal");
                    break;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    // Sender упал — тоже останавливаемся
                    error!("Engine stop channel closed unexpectedly");
                    break;
                }
            }

            // Тик
            let commands = cmd_bus.drain_sorted();
            engine.tick(commands);

            // Снапшот
            let entities_view: Vec<EntityView> = engine
                .snapshot_entities()
                .into_iter()
                .map(|snap| EntityView {
                    guid: snap.guid.map(|g| g.to_string()).unwrap_or_default(),
                    x: snap.x,
                    y: snap.y,
                    glyph: snap.glyph,
                    color: format!("#{:06X}", snap.color_rgb),
                })
                .collect();

            let snapshots = engine.snapshot_entities();
            if let Ok(mut state) = api_state_engine.lock() {
                state.tick = engine.current_tick().0;
                state.entity_count = snapshots.len() as u32;
                state.entities = snapshots
                    .into_iter()
                    .map(|s| ApiEntity {
                        guid: s.guid.map(|g| g.to_string()).unwrap_or_default(),
                        x: s.x,
                        y: s.y,
                        glyph: s.glyph,
                        color: format!("#{:06X}", s.color_rgb),
                    })
                    .collect();
            }

            let packet = ServerPacket::Snapshot {
                tick: engine.current_tick().0,
                entities: entities_view,
            };
            let _ = snapshot_tx.send(packet);

            // Tick rate
            let elapsed = start.elapsed();
            if elapsed < tick_rate {
                std::thread::sleep(tick_rate - elapsed);
            }
        }

        // Корректное завершение ПОСЛЕ выхода из loop
        engine.shutdown();
        info!("Engine thread finished");
    });

    // --- Сетевой слой ---
    let net_handle = tokio::spawn(async move {
        cd_net::run_server(
            8080,
            cmd_sender,
            snapshot_tx_net,
            telemetry_tx,
            net_stop_rx,
            api_state_net,
            reload_cb_net,
        )
        .await;
        info!("Network finished");
    });

    // --- Ожидание сигнала (Ctrl+C или SIGTERM) ---
    wait_for_shutdown_signal().await;
    info!("🛑 Shutdown signal received, stopping gracefully...");

    // Останавливаем сначала сеть (перестаём принимать команды)
    let _ = net_stop_tx.send(());

    // Затем движок (дообрабатывает текущий тик и сохраняет данные)
    let _ = engine_stop_tx.send(());

    // Ждём завершения обоих
    net_handle.await.expect("Network task panicked");
    engine_handle.join().expect("Engine thread panicked");

    info!("✅ Cognitive Dungeon stopped cleanly");
}

async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    // На не-unix платформах SIGTERM не поддерживается — ждём только Ctrl+C
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
