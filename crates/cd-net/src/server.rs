use crate::api::{ReloadCallback, SharedApiState, handler_get_state, handler_reload_data};
use crate::error::NetResult;
use crate::protocol::{ClientPacket, ServerPacket};
use crate::session::Session;
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
use tokio::sync::{broadcast as tbroadcast, mpsc, oneshot};
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
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // 1. Sender Task (не изменился)
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 2. Snapshot Task (не изменился)
    let tx_snap = tx.clone();
    let mut rx_snap = state.snapshot_tx.subscribe();
    tokio::spawn(async move {
        while let Ok(pkt) = rx_snap.recv().await {
            let json = serde_json::to_string(&pkt).unwrap();
            if tx_snap.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // 3. Создаем сессию и роутер
    let session_id = 1; // Тут можно генерировать уникальный ID подключения
    let session = Session::new(session_id, tx);
    let router = crate::Router::new(state.cmd_tx.clone());

    // 4. Главный цикл чтения
    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            // Обработка одной строки: парсинг + роутинг + обработка ошибок
            let res: NetResult<()> = async {
                let packet: ClientPacket = serde_json::from_str(&text)?;
                router.dispatch(session.clone(), packet).await?;
                Ok(())
            }
            .await;

            // Централизованное логирование ошибок
            if let Err(e) = res {
                tracing::warn!("Handler error for session {}: {}", session_id, e);
                // Опционально: можно отправить ошибку клиенту
                // let _ = session.send_json(&ServerPacket::Error(e.to_string())).await;
            }
        }
    }
}
