use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use cd_telemetry::EngineEvent;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

pub type TelemetryState = Arc<broadcast::Sender<EngineEvent>>;

pub async fn telemetry_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<TelemetryState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_telemetry_socket(socket, state))
}

async fn handle_telemetry_socket(socket: WebSocket, tx: TelemetryState) {
    let mut rx = tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    info!("SDK connected to /telemetry");

    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Ждём закрытия соединения со стороны клиента
    while let Some(Ok(_)) = receiver.next().await {}

    send_task.abort();
    info!("SDK disconnected from /telemetry");
}
