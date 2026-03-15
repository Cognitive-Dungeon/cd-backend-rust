use anyhow::Result;
use cd_core::{ObjectGuid, WorldPos};
use cd_data::json::{JsonEntityRepository, JsonWorldRepository};
use cd_engine::{BroadcastSink, CommandBus, Engine, EngineBuilder};
use cd_map::{Chunk, Tile, TileFlags};
use cd_net::protocol::{EntityView, ServerPacket};
use cd_net::{ApiEntity, ApiState, ReloadCallback, SharedApiState};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle as TokioJoinHandle;
use tracing::{Level, error, info};

/// Главная структура приложения, которая управляет его жизненным циклом.
pub struct Application {
    engine_thread: JoinHandle<()>,
    network_task: TokioJoinHandle<()>,
    engine_stop_tx: oneshot::Sender<()>,
    net_stop_tx: oneshot::Sender<()>,
}

impl Application {
    /// Запускает приложение и ждет сигнала о завершении.
    pub async fn run(self) -> Result<()> {
        info!("✅ Cognitive Dungeon is running. Press Ctrl+C to exit.");
        wait_for_shutdown_signal().await;
        self.shutdown().await
    }

    /// Корректно останавливает все компоненты.
    async fn shutdown(self) -> Result<()> {
        info!("🛑 Shutdown signal received, stopping gracefully...");

        // Сначала останавливаем сеть (перестаем принимать команды)
        let _ = self.net_stop_tx.send(());
        // Затем движок (он доработает последний тик и сохранится)
        let _ = self.engine_stop_tx.send(());

        // Ждем завершения обоих потоков/задач
        self.network_task.await?;
        self.engine_thread.join().expect("Engine thread panicked");

        info!("✅ Cognitive Dungeon stopped cleanly.");
        Ok(())
    }
}

/// Builder для создания и конфигурации приложения.
pub struct ApplicationBuilder {
    data_path: String,
    depot_filename: String,
    port: u16,
    tick_rate_ms: u64,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self {
            data_path: "./data".to_string(),
            depot_filename: "game.cdb".to_string(),
            port: 8080,
            tick_rate_ms: 50, // 20 TPS
        }
    }

    /// Собирает все компоненты, связывает их и возвращает готовое приложение.
    pub fn build(self) -> Result<Application> {
        // --- 1. Каналы для связи компонентов ---
        let (mut cmd_bus, cmd_sender) = CommandBus::new(1024);
        let (snapshot_tx, _) = broadcast::channel::<ServerPacket>(16);
        let (telemetry_sink, telemetry_tx) = BroadcastSink::new(256);
        let (engine_stop_tx, engine_stop_rx) = oneshot::channel::<()>();
        let (net_stop_tx, net_stop_rx) = oneshot::channel::<()>();

        // --- 2. Разделяемое состояние (State) ---
        let api_state: SharedApiState = Arc::new(Mutex::new(ApiState::default()));
        let game_data: Arc<RwLock<Option<cd_engine::Depot>>> = Arc::new(RwLock::new(None));

        // --- 3. Репозитории и Горячая перезагрузка ---
        let depot_path =
            std::path::PathBuf::from(format!("{}/{}", self.data_path, self.depot_filename));
        let world_repo = Arc::new(JsonWorldRepository::new(&self.data_path)?);
        let entity_repo = Arc::new(JsonEntityRepository::new(&self.data_path)?);

        let reload_cb: ReloadCallback = {
            let game_data_api = game_data.clone();
            let depot_path_api = depot_path.clone();
            Arc::new(tokio::sync::Mutex::new(Box::new(
                move || match cd_engine::Depot::load(&depot_path_api) {
                    Ok(depot) => *game_data_api.write().unwrap() = Some(depot),
                    Err(e) => tracing::error!("API reload failed: {}", e),
                },
            )))
        };

        // --- 4. Запуск потоков/задач ---
        let engine_thread = spawn_engine_thread(
            engine_stop_rx,
            cmd_bus,
            snapshot_tx.clone(),
            Arc::new(telemetry_sink),
            world_repo,
            entity_repo,
            game_data.clone(),
            api_state.clone(),
            depot_path,
            Duration::from_millis(self.tick_rate_ms),
        );

        let network_task = tokio::spawn(cd_net::run_server(
            self.port,
            cmd_sender,
            snapshot_tx,
            telemetry_tx,
            net_stop_rx,
            api_state,
            reload_cb,
        ));

        Ok(Application {
            engine_thread,
            network_task,
            engine_stop_tx,
            net_stop_tx,
        })
    }
}

/// Главная функция, которая запускает все и возвращает готовое приложение.
pub async fn run() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("🚀 Booting Cognitive Dungeon...");

    ApplicationBuilder::new().build()?.run().await
}

// -----------------------------------------------------------------------------
// - Логика потока движка
// -----------------------------------------------------------------------------
fn spawn_engine_thread(
    mut stop_rx: oneshot::Receiver<()>,
    mut cmd_bus: CommandBus,
    snapshot_tx: broadcast::Sender<ServerPacket>,
    telemetry_sink: Arc<BroadcastSink>,
    world_repo: Arc<JsonWorldRepository>,
    entity_repo: Arc<JsonEntityRepository>,
    game_data: Arc<RwLock<Option<cd_engine::Depot>>>,
    api_state: SharedApiState,
    depot_path: std::path::PathBuf,
    tick_rate: Duration,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut engine = EngineBuilder::new()
            .telemetry(telemetry_sink)
            .world_repo(world_repo)
            .entity_repo(entity_repo)
            .world_seed(0xDEAD_CAFE_BABE_1337)
            .build();

        // Первичная загрузка данных
        if depot_path.exists() {
            match cd_engine::Depot::load(&depot_path) {
                Ok(depot) => *game_data.write().unwrap() = Some(depot),
                Err(e) => error!("Failed to load initial depot: {}", e),
            }
        }

        // Следим за файлом для hot-reload
        let game_data_watcher = game_data.clone();
        let _watcher = cd_engine::watcher::spawn_depot_watcher(depot_path, move |path| {
            match cd_engine::Depot::load(path) {
                Ok(depot) => *game_data_watcher.write().unwrap() = Some(depot),
                Err(e) => error!("Hot reload from watcher failed: {}", e),
            }
        });

        engine.register_system("movement", cd_engine::systems::movement::run);

        // TODO: Перенести в сценарий/конфиг
        setup_initial_world(&mut engine);

        // --- Главный цикл движка ---
        loop {
            if stop_rx.try_recv().is_ok() {
                info!("Engine received stop signal.");
                break;
            }

            let start = std::time::Instant::now();

            engine.tick(cmd_bus.drain_sorted());

            update_api_state(&engine, &api_state);
            send_snapshot(&engine, &snapshot_tx);

            let elapsed = start.elapsed();
            if elapsed < tick_rate {
                std::thread::sleep(tick_rate - elapsed);
            }
        }

        engine.shutdown();
        info!("Engine thread finished.");
    })
}

fn setup_initial_world(engine: &mut Engine) {
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
}

fn update_api_state(engine: &Engine, api_state: &SharedApiState) {
    let snapshots = engine.snapshot_entities();
    if let Ok(mut state) = api_state.lock() {
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
}

fn send_snapshot(engine: &Engine, snapshot_tx: &broadcast::Sender<ServerPacket>) {
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

    let packet = ServerPacket::Snapshot {
        tick: engine.current_tick().0,
        entities: entities_view,
    };
    let _ = snapshot_tx.send(packet);
}

// -----------------------------------------------------------------------------
// - Обработчик сигналов (Ctrl+C, SIGTERM)
// -----------------------------------------------------------------------------
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

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
