use crate::api::{ReloadCallback, SharedApiState, handler_get_state, handler_reload_data};
use crate::protocol::{ClientPacket, ServerPacket};
use crate::telemetry::{TelemetryState, telemetry_ws_handler};
use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    routing::post,
};
use cd_core::{ObjectGuid, WorldPos};
use cd_engine::{CommandSender, InputCmd};
use cd_telemetry::EngineEvent;
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast as tbroadcast, oneshot};
use tracing::{error, info, warn};

/// Контекст, доступный всем обработчикам
struct AppState {
    /// Канал для отправки команд в Движок
    cmd_tx: CommandSender,
    /// Канал для получения обновлений мира (подписка)
    snapshot_tx: tbroadcast::Sender<ServerPacket>,
}

pub async fn run_server(
    port: u16,
    cmd_tx: CommandSender,
    snapshot_tx: tbroadcast::Sender<ServerPacket>,
    telemetry_tx: tbroadcast::Sender<EngineEvent>,
    stop_rx: oneshot::Receiver<()>,
    api_state: SharedApiState,
    reload_cb: ReloadCallback,
) {
    let game_state = Arc::new(AppState {
        cmd_tx,
        snapshot_tx,
    });
    let telemetry_state: TelemetryState = Arc::new(telemetry_tx);

    let app = Router::new()
        .merge(
            Router::new()
                .route("/ws", get(ws_handler))
                .with_state(game_state),
        )
        .merge(
            Router::new()
                .route("/telemetry", get(telemetry_ws_handler))
                .with_state(telemetry_state),
        )
        .merge(
            Router::new()
                .route("/api/state", get(handler_get_state))
                .with_state(api_state),
        )
        .merge(
            Router::new()
                .route("/api/reload-data", post(handler_reload_data))
                .with_state(reload_cb),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🌐 Network listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // graceful shutdown: axum ждёт сигнала и закрывает существующие соединения
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = stop_rx.await;
            info!("Network layer shutting down");
        })
        .await
        .unwrap();
}

/// HTTP Handshake -> WebSocket Upgrade
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Логика одного клиента
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut my_guid: Option<ObjectGuid> = None;

    // Подписываемся на снапшоты (Broadcast)
    let mut rx_snapshot = state.snapshot_tx.subscribe();

    // Spawn задачи на отправку снапшотов клиенту
    let send_task = tokio::spawn(async move {
        while let Ok(packet) = rx_snapshot.recv().await {
            // Сериализуем в JSON
            let json = serde_json::to_string(&packet).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break; // Клиент отвалился
            }
        }
    });

    // Цикл чтения сообщений от клиента
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            // 1. Парсинг
            let packet: ClientPacket = match serde_json::from_str(&text) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Invalid JSON: {}", e);
                    continue;
                }
            };

            // 2. Обработка (Auth или Command)
            match packet {
                ClientPacket::Login { token } => {
                    // TODO: Реальная авторизация
                    // Пока генерируем фейковый GUID на основе длины токена для теста
                    let mock_id = token.len() as u32;
                    let guid = ObjectGuid::new(1, 1, 1, mock_id);
                    my_guid = Some(guid);

                    info!("Client logged in: {:?}", guid);

                    // Уведомляем движок (в реальной системе это тоже InputCmd::Login)
                    // Но пока мы считаем, что логин прошел
                    // Отправляем InputCmd::Spawn (если бы он был)
                    // Для простоты фазы 4: считаем, что движок сам заспавнит по запросу,
                    // но здесь мы просто запомнили GUID сессии.
                }
                ClientPacket::Move { x, y } => {
                    if let Some(guid) = my_guid {
                        // Транслируем DTO -> Engine Command
                        let cmd = InputCmd::Move {
                            entity_guid: guid,
                            target: WorldPos::new(x, y, 0),
                        };

                        // Отправляем в движок (Non-blocking)
                        if let Err(_) = state.cmd_tx.send(cmd).await {
                            error!("Engine is dead");
                            break;
                        }
                    } else {
                        warn!("Command before login ignored");
                    }
                }
            }
        }
    }

    send_task.abort();
    info!("Client disconnected {:?}", my_guid);
}
