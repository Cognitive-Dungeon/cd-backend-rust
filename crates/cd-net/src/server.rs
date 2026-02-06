use crate::protocol::{ClientPacket, ServerPacket};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use cd_core::{ObjectGuid, WorldPos};
use cd_engine::InputCmd;
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

/// Контекст, доступный всем обработчикам
struct AppState {
    /// Канал для отправки команд в Движок
    cmd_tx: mpsc::Sender<InputCmd>,
    /// Канал для получения обновлений мира (подписка)
    snapshot_tx: broadcast::Sender<ServerPacket>,
}

pub async fn run_server(
    port: u16,
    cmd_tx: mpsc::Sender<InputCmd>,
    snapshot_tx: broadcast::Sender<ServerPacket>,
) {
    let state = Arc::new(AppState {
        cmd_tx,
        snapshot_tx,
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🌐 Network listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// HTTP Handshake -> WebSocket Upgrade
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
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