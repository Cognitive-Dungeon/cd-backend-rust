// crates/cd-app/src/app.rs
use anyhow::Result;
use bevy_ecs::schedule::IntoScheduleConfigs;
use cd_data::json::{JsonEntityRepository, JsonWorldRepository};
use cd_data::provider::RonDataProvider;
use cd_engine::{BroadcastSink, CommandBus, Engine, EngineBuilder, InputCmd};
use cd_net::protocol::{OutboundMessage, ServerPacket, TileView};
use cd_net::snapshot::SnapshotBuilder;
use cd_net::{ApiEntity, ApiState, ReloadCallback, SharedApiState};
use std::sync::{Arc, Mutex};
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
    port: u16,
    tick_rate_ms: u64,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self {
            data_path: "./data".to_string(),
            port: 8080,
            tick_rate_ms: 50, // 20 TPS
        }
    }

    pub fn build(self) -> Result<Application> {
        // --- 1. ТРУБЫ (Каналы связи) ---
        let (cmd_bus, cmd_sender) = CommandBus::new(1024);
        let (outbound_tx, _) = broadcast::channel::<cd_net::protocol::OutboundMessage>(16);
        let (telemetry_sink, telemetry_tx) = BroadcastSink::new(256);
        let (engine_stop_tx, engine_stop_rx) = oneshot::channel();
        let (net_stop_tx, net_stop_rx) = oneshot::channel();

        // --- 2. ОБЩЕЕ СОСТОЯНИЕ И ДАННЫЕ ---
        let api_state: SharedApiState = Arc::new(Mutex::new(ApiState::default()));
        // Создаем наш новый стейтлесс-провайдер (никаких RwLock!)
        let data_provider = Arc::new(RonDataProvider::new(&self.data_path));

        // --- 3. ИГРОВЫЕ ДАННЫЕ (Репозитории) ---
        let world_repo = Arc::new(JsonWorldRepository::new(&self.data_path)?);
        let entity_repo = Arc::new(JsonEntityRepository::new(&self.data_path)?);

        // Колбэк для REST API
        let reload_cb: ReloadCallback = {
            let cmd_tx = cmd_sender.clone();
            Arc::new(tokio::sync::Mutex::new(Box::new(move || {
                let sender = cmd_tx.clone();
                // Запускаем асинхронную отправку команды в фоне
                tokio::spawn(async move {
                    if let Err(e) = sender.send(InputCmd::ReloadData).await {
                        tracing::error!("Failed to send ReloadData command: {}", e);
                    }
                });
            })))
        };

        // --- 4. ЗАПУСК ДВИЖКА (Изолированный поток ОС) ---
        let engine_thread = {
            let outbound_tx = outbound_tx.clone();
            let api_state = api_state.clone();
            let provider_for_engine = data_provider.clone();

            std::thread::spawn(move || {
                let engine = build_engine(
                    Arc::new(telemetry_sink),
                    world_repo,
                    entity_repo,
                    provider_for_engine,
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
            outbound_tx.subscribe(),
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
    data_provider: Arc<dyn cd_data::provider::DataProvider>,
) -> Engine {
    let mut engine = EngineBuilder::new()
        .telemetry(telemetry_sink)
        .world_repo(world_repo)
        .entity_repo(entity_repo)
        .world_seed(0xDEAD_CAFE_BABE_1337)
        .data_provider(data_provider)
        .build();

    // 1. Загрузка данных
    tracing::info!("Loading game data via DataProvider...");
    engine.rebuild_cache();

    // 2. Регистрация систем
    engine.schedule.add_systems(
        (
            cd_engine::systems::input::handle_input_system,
            cd_engine::systems::turn::npc_ai_system,
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

    // 3. ОТПРАВКА КАРТЫ (Раз в секунду для теста)
    if tick.is_multiple_of(20) {
        let chunk_pos = cd_core::WorldPos::new(0, 0, 0);
        let chunk_snap = SnapshotBuilder::build_chunk(&engine.world, chunk_pos);

        let palette_view: Vec<TileView> = chunk_snap
            .palette
            .into_iter()
            .map(|g| TileView {
                glyph: g.to_char(),
                color: g.hex_color(),
            })
            .collect();

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
