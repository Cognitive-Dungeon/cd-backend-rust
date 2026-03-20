use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use cd_telemetry::EngineEvent;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

pub type TelemetryBus = Arc<broadcast::Sender<EngineEvent>>;

/// Соединение SDK-клиента, который слушает события движка в реальном времени.
///
/// Только исходящий поток: клиент ничего не отправляет, только получает.
pub struct TelemetryConnection {
    /// Подписка на события движка
    events: broadcast::Receiver<EngineEvent>,
}

impl TelemetryConnection {
    pub fn subscribe(bus: &TelemetryBus) -> Self {
        Self {
            events: bus.subscribe(),
        }
    }

    /// Запускает жизненный цикл до отключения клиента.
    pub async fn run(self, socket: WebSocket) {
        let (ws_outgoing, ws_incoming) = socket.split();

        // Поток 1: пересылаем события движка клиенту
        let send_task = tokio::spawn(Self::forward_events(self.events, ws_outgoing));

        // Поток 2 (основной): ждём закрытия со стороны клиента
        Self::wait_for_disconnect(ws_incoming).await;

        send_task.abort();
        tracing::info!("SDK disconnected from /telemetry");
    }

    /// Сериализует события движка и шлёт их клиенту.
    async fn forward_events(
        mut events: broadcast::Receiver<EngineEvent>,
        mut ws_outgoing: impl SinkExt<Message, Error = axum::Error> + Unpin,
    ) {
        tracing::info!("SDK connected to /telemetry");
        while let Ok(event) = events.recv().await {
            let Ok(json) = serde_json::to_string(&event) else {
                continue;
            };
            if ws_outgoing.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    }

    /// Держит соединение открытым пока клиент не закроет его со своей стороны.
    async fn wait_for_disconnect(
        mut ws_incoming: impl StreamExt<Item = Result<Message, axum::Error>> + Unpin,
    ) {
        while let Some(Ok(_)) = ws_incoming.next().await {}
    }
}

/// Axum-хэндлер: апгрейдит HTTP → WebSocket и запускает TelemetryConnection.
pub async fn telemetry_ws_handler(
    ws: WebSocketUpgrade,
    State(bus): State<TelemetryBus>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        TelemetryConnection::subscribe(&bus).run(socket).await;
    })
}
