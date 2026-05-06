// crates/cd-app/src/app.rs
use anyhow::Result;
use bevy_ecs::schedule::IntoScheduleConfigs;
use cd_data::json::{JsonEntityRepository, JsonWorldRepository};
use cd_engine::{BroadcastSink, CommandBus, Engine, EngineBuilder};
use cd_net::protocol::{EntityView, OutboundMessage, ServerPacket, TileView};
use cd_net::snapshot::SnapshotBuilder;
use cd_net::{ApiEntity, ApiState, ReloadCallback, SharedApiState};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle as TokioJoinHandle;
use tracing::{Level, info};

/// Главная структура приложения, которая управляет его жизненным циклом.
pub struct Application {
    engine_thread: JoinHandle<()>,
    network_task: TokioJoinHandle<()>,
    engine_stop_tx: oneshot::Sender<()>,
    net_stop_tx: oneshot::Sender<()>,
}

impl Application {
    pub async fn run(self) -> Result<()> {
        info!("✅ Cognitive Dungeon is running. Press Ctrl+C to exit.");
        wait_for_shutdown_signal().await;
        self.shutdown().await
    }

    async fn shutdown(self) -> Result<()> {
        info!("🛑 Shutdown signal received, stopping gracefully...");

        // 1. Сначала останавливаем сеть (перестаем принимать новые команды)
        let _ = self.net_stop_tx.send(());
        self.network_task.await?;

        // 2. Затем движок (он доработает последний тик и сохранит мир)
        let _ = self.engine_stop_tx.send(());
        self.engine_thread.join().expect("Engine thread panicked");

        info!("✅ Cognitive Dungeon stopped cleanly.");
        Ok(())
    }
}

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

    pub fn build(self) -> Result<Application> {
        // --- 1. ТРУБЫ (Каналы связи) ---
        // Команды от Сети -> в Движок
        let (cmd_bus, cmd_sender) = CommandBus::new(1024);

        // Сообщения от Движка -> в Сеть
        let (outbound_tx, _) = broadcast::channel::<cd_net::protocol::OutboundMessage>(16);

        // Телеметрия от Движка -> в Сеть (SDK)
        let (telemetry_sink, telemetry_tx) = BroadcastSink::new(256);

        // Сигналы остановки
        let (engine_stop_tx, engine_stop_rx) = oneshot::channel();
        let (net_stop_tx, net_stop_rx) = oneshot::channel();

        // --- 2. ОБЩЕЕ СОСТОЯНИЕ (Только для REST API) ---
        let api_state: SharedApiState = Arc::new(Mutex::new(ApiState::default()));
        let game_data: Arc<RwLock<Option<cd_engine::Depot>>> = Arc::new(RwLock::new(None));

        // --- 3. ИГРОВЫЕ ДАННЫЕ (Репозитории) ---
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

        // --- 4. ЗАПУСК ДВИЖКА (Изолированный поток ОС) ---
        let engine_thread = {
            let game_data = game_data.clone();
            let outbound_tx = outbound_tx.clone();
            let api_state = api_state.clone();
            let depot_path = depot_path.clone();

            std::thread::spawn(move || {
                let engine = build_engine(
                    Arc::new(telemetry_sink),
                    world_repo,
                    entity_repo,
                    game_data,
                    &depot_path,
                );
                run_engine_loop(
                    engine,
                    engine_stop_rx,
                    cmd_bus,
                    outbound_tx,
                    api_state,
                    Duration::from_millis(self.tick_rate_ms),
                );
            })
        };

        // --- 5. ЗАПУСК СЕТИ (Пул асинхронных задач) ---
        let network_task = tokio::spawn(cd_net::run_server(
            self.port,
            cmd_sender,
            outbound_tx.subscribe(), // Передаем Receiver! Сеть только слушает.
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

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("🚀 Booting Cognitive Dungeon...");
    ApplicationBuilder::new().build()?.run().await
}

// =============================================================================
// = ИЗОЛИРОВАННЫЙ ПОТОК ДВИЖКА
// =============================================================================

/// Инициализация движка: загрузка данных, регистрация систем, генерация мира.
fn build_engine(
    telemetry_sink: Arc<BroadcastSink>,
    world_repo: Arc<JsonWorldRepository>,
    entity_repo: Arc<JsonEntityRepository>,
    game_data: Arc<RwLock<Option<cd_engine::Depot>>>,
    depot_path: &std::path::Path,
) -> Engine {
    let mut engine = EngineBuilder::new()
        .telemetry(telemetry_sink)
        .world_repo(world_repo)
        .entity_repo(entity_repo)
        .world_seed(0xDEAD_CAFE_BABE_1337)
        .game_data(game_data.clone())
        .build();

    // 1. Загрузка Depot
    tracing::info!("Trying to load Depot from {:?}", depot_path);
    if depot_path.exists() {
        match cd_engine::Depot::load(depot_path) {
            Ok(depot) => {
                *game_data.write().unwrap() = Some(depot);
                engine.rebuild_cache();
                tracing::info!("Depot loaded and cache rebuilt successfully!");
            }
            Err(e) => tracing::error!("Failed to parse game.cdb: {}", e),
        }
    } else {
        tracing::error!("Depot file NOT FOUND at {:?}", depot_path);
    }

    // 2. Регистрация систем
    engine.schedule.add_systems(
        (
            cd_engine::systems::input::handle_input_system,
            cd_engine::systems::turn::combat_turn_system,
            cd_engine::systems::movement::movement_system,
            cd_engine::systems::spell::spell_system,
        )
            .chain(),
    );

    // 3. Генерация тестового мира
    engine.generate_test_world();
    engine.spawn_test_mob();

    engine
}

/// Игровой цикл: тикает движок, публикует состояние, слушает сигнал остановки.
fn run_engine_loop(
    mut engine: Engine,
    mut stop_rx: oneshot::Receiver<()>,
    mut cmd_bus: CommandBus,
    outbound_tx: broadcast::Sender<OutboundMessage>,
    api_state: SharedApiState,
    tick_rate: Duration,
) {
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        let start = Instant::now();

        engine.tick(cmd_bus.drain_sorted());
        publish_state(&mut engine, &api_state, &outbound_tx);

        let elapsed = start.elapsed();
        if elapsed < tick_rate {
            std::thread::sleep(tick_rate - elapsed);
        }
    }

    engine.shutdown();
    info!("Engine thread finished.");
}

// -----------------------------------------------------------------------------
// - АДАПТЕРЫ (Клеевой код между Движком и Сетью)
// -----------------------------------------------------------------------------

/// Транслирует внутренний формат движка (EntitySnapshot) в DTO для Сети и REST API
fn publish_state(
    engine: &mut Engine,
    api_state: &SharedApiState,
    outbound_tx: &broadcast::Sender<cd_net::protocol::OutboundMessage>,
) {
    let snapshots = cd_net::snapshot::SnapshotBuilder::build_entities(&mut engine.world);
    let tick = engine.current_tick().0;

    // 1. REST API (для админки / отладки)
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

    // 2. WebSocket рассылка
    // Если никто не подключен (канал пуст), .send() вернет ошибку, которую мы просто игнорируем (_)
    // 1. ОТПРАВКА СУЩНОСТЕЙ (Каждый тик)
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

    let packet = ServerPacket::Snapshot {
        tick,
        entities: entities_view,
    };
    let _ = outbound_tx.send(OutboundMessage::broadcast(packet));

    // 2. ОТПРАВКА КАРТЫ (Раз в секунду для теста)
    if tick.is_multiple_of(20) {
        // Берем чанк 0,0
        let chunk_pos = cd_core::WorldPos::new(0, 0, 0);
        let chunk_snap = SnapshotBuilder::build_chunk(&engine.world, chunk_pos);

        // Перегоняем палитру в сетевой формат TileView
        let palette_view: Vec<TileView> = chunk_snap
            .palette
            .into_iter()
            .map(|g| TileView {
                glyph: g.to_char(),
                color: g.hex_color(),
            })
            .collect();

        // Отправляем! Обрати внимание на опечатку в protocol.rs (pallete с двумя l и одной t)
        let chunk_packet = ServerPacket::MapChunk {
            x: chunk_snap.chunk_x,
            y: chunk_snap.chunk_y,
            palette: palette_view,
            indices: chunk_snap.indices,
        };

        let _ = outbound_tx.send(OutboundMessage::broadcast(chunk_packet));
    }
}

// -----------------------------------------------------------------------------
// - СИГНАЛЫ ОС
// -----------------------------------------------------------------------------
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap();
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
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
