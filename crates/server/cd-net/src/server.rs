use crate::api::{self, ReloadCallback, SharedApiState};
use crate::connection::Connection;
use crate::manager::{ConnectionManager, SharedManager};
use crate::protocol::{ClientPacket, OutboundMessage, ServerPacket};
use crate::session::Session;
use crate::telemetry::{TelemetryBus, telemetry_ws_handler};
use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
};
use cd_engine::CommandSender;
use cd_telemetry::EngineEvent;
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{broadcast as tbroadcast, mpsc, oneshot};
use tracing::{info, warn};

// Глобальный счетчик сессий
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

struct AppState {
    cmd_tx: CommandSender,
    manager: SharedManager,
    router: crate::Router,
}

pub async fn run_server(
    port: u16,
    cmd_tx: CommandSender,
    mut outbound_rx: tbroadcast::Receiver<OutboundMessage>,
    telemetry_tx: tbroadcast::Sender<EngineEvent>,
    stop_rx: oneshot::Receiver<()>,
    api_state: SharedApiState,
    reload_cb: ReloadCallback,
) {
    let manager = ConnectionManager::new();
    let router = crate::Router::new(cmd_tx.clone());

    let app_state = Arc::new(AppState {
        cmd_tx,
        manager: manager.clone(),
        router,
    });

    let telemetry_bus: TelemetryBus = Arc::new(telemetry_tx);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .route("/telemetry", get(telemetry_ws_handler))
        .with_state(telemetry_bus)
        .route("/api/state", get(api::handlers::get_state))
        .with_state(api_state)
        .route("/api/reload-data", post(api::handlers::reload_data))
        .with_state(reload_cb);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🌐 Network listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // --- ДИСПЕТЧЕР ДВИЖКА ---
    // Слушает пакеты от движка и раскидывает их через ConnectionManager
    let manager_for_dispatch = manager.clone();
    tokio::spawn(async move {
        while let Ok(msg) = outbound_rx.recv().await {
            match msg.target {
                crate::protocol::OutboundTarget::Broadcast => {
                    manager_for_dispatch.broadcast(msg.packet).await;
                }
                crate::protocol::OutboundTarget::Single(guid) => {
                    manager_for_dispatch.send_to_agent(guid, msg.packet).await;
                }
                crate::protocol::OutboundTarget::Multiple(guids) => {
                    manager_for_dispatch
                        .send_to_agents(&guids, msg.packet)
                        .await;
                }
            }
        }
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = stop_rx.await;
            info!("Network layer shutting down");
        })
        .await
        .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let conn = Connection::accept(session_id, state.manager.clone());
        conn.run(socket, &state.router).await;
    })
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Канал для отправки данных из Менеджера в этот Сокет
    let (tx_out, mut rx_out) = mpsc::channel::<ServerPacket>(state.manager.config().buffer_cap);

    state.manager.add_session(session_id, tx_out);
    let session = Session::new(session_id, state.manager.clone());

    // ЗАДАЧА 1: ПИСАТЕЛЬ (Берет структуры, сериализует в JSON, шлет в сокет)
    let write_task = tokio::spawn(async move {
        while let Some(packet) = rx_out.recv().await {
            if let Ok(json) = serde_json::to_string(&packet)
                && ws_sender.send(Message::Text(json.into())).await.is_err()
            {
                break;
            }
        }
    });

    // ЗАДАЧА 2: ЧИТАТЕЛЬ (Берет JSON, десериализует, отдает Роутеру)
    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            let res: crate::error::NetResult<()> = async {
                let packet: ClientPacket = serde_json::from_str(&text)?;
                state.router.dispatch(session.clone(), packet).await?;
                Ok(())
            }
            .await;

            if let Err(e) = res {
                warn!("Handler error for session {}: {}", session_id, e);
            }
        }
    }

    // Если мы вышли из цикла (клиент закрыл вкладку / оборвался инет):
    write_task.abort();
    // Менеджер сам запустит Reconnect Window, если к сессии был привязан агент
    state.manager.remove_session(session_id);
}
