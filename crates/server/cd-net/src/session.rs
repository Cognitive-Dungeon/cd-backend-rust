use crate::error::{NetError, NetResult};
use axum::extract::ws::Message;
use cd_core::ObjectGuid;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// Состояние сессии, защищенное RwLock (чтобы менять GUID при логине)
#[derive(Debug, Default)]
pub struct SessionState {
    pub guid: Option<ObjectGuid>,
    pub username: Option<String>,
}

/// Хэндл сессии, который дешево клонировать и передавать в хендлеры
#[derive(Clone)]
pub struct Session {
    id: u64, // Внутренний ID соединения (для логов)
    sender: mpsc::Sender<Message>,
    state: Arc<RwLock<SessionState>>,
}

impl Session {
    pub fn new(id: u64, sender: mpsc::Sender<Message>) -> Self {
        Self {
            id,
            sender,
            state: Arc::new(RwLock::new(SessionState::default())),
        }
    }

    pub async fn send_json(&self, value: &impl serde::Serialize) -> NetResult<()> {
        let text = serde_json::to_string(value)?;
        self.sender
            .send(Message::Text(text.into()))
            .await
            .map_err(|_| NetError::Ws(axum::Error::new("Sender dropped"))) // Упрощение
    }

    pub async fn set_authenticated(&self, guid: ObjectGuid, name: String) {
        let mut state = self.state.write().await;
        state.guid = Some(guid);
        state.username = Some(name);
    }

    pub async fn get_guid(&self) -> Option<ObjectGuid> {
        self.state.read().await.guid
    }

    // Хелпер для middleware проверки
    pub async fn require_guid(&self) -> NetResult<ObjectGuid> {
        self.get_guid().await.ok_or(NetError::Unauthorized)
    }
}
