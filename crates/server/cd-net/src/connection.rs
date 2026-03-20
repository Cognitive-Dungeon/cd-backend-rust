use crate::manager::SharedManager;
use crate::protocol::{ClientPacket, ServerPacket};
use crate::router::Router;
use crate::session::Session;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

/// Одно активное WebSocket-соединение.
///
/// Внутри два параллельных потока:
/// - `outbox`: ждёт пакеты от движка → сериализует → шлёт клиенту
/// - основной: читает клиента → десериализует → отдаёт роутеру
pub struct Connection {
    session: Session,
    /// Канал: движок/менеджер кладут сюда пакеты для этого клиента
    outbox: mpsc::Receiver<ServerPacket>,
}

impl Connection {
    /// Регистрирует новое соединение в менеджере и возвращает готовый Connection.
    pub fn accept(session_id: u64, manager: SharedManager) -> Self {
        // Создаём "трубу": менеджер пишет в inbox_tx, мы читаем из outbox
        let buffer_cap = manager.config().buffer_cap;
        let (inbox_tx, outbox) = mpsc::channel::<ServerPacket>(buffer_cap);

        manager.add_session(session_id, inbox_tx);
        let session = Session::new(session_id, manager);

        Self { session, outbox }
    }

    /// Запускает жизненный цикл соединения до отключения клиента.
    pub async fn run(self, socket: WebSocket, router: &Router) {
        let (ws_outgoing, ws_incoming) = socket.split();

        // Поток 1: пересылаем пакеты от движка клиенту
        let send_task = tokio::spawn(Self::forward_to_client(self.outbox, ws_outgoing));

        // Поток 2 (основной): читаем клиента и диспатчим команды
        Self::receive_from_client(ws_incoming, &self.session, router).await;

        // Клиент отключился — останавливаем отправку
        send_task.abort();
        self.session.on_disconnect().await;
    }

    /// Берёт пакеты из внутренней очереди и отправляет их клиенту как JSON.
    async fn forward_to_client(
        mut outbox: mpsc::Receiver<ServerPacket>,
        mut ws_outgoing: impl SinkExt<Message, Error = axum::Error> + Unpin,
    ) {
        while let Some(packet) = outbox.recv().await {
            let Ok(json) = serde_json::to_string(&packet) else {
                continue;
            };
            if ws_outgoing.send(Message::Text(json.into())).await.is_err() {
                break; // клиент пропал
            }
        }
    }

    /// Читает сообщения от клиента и передаёт их роутеру.
    async fn receive_from_client(
        mut ws_incoming: impl StreamExt<Item = Result<Message, axum::Error>> + Unpin,
        session: &Session,
        router: &Router,
    ) {
        while let Some(Ok(Message::Text(text))) = ws_incoming.next().await {
            let result: crate::error::NetResult<()> = async {
                let packet: ClientPacket = serde_json::from_str(&text)?;
                router.dispatch(session.clone(), packet).await
            }
            .await;

            if let Err(e) = result {
                tracing::warn!("Handler error for session {}: {}", session.id, e);
            }
        }
    }
}
