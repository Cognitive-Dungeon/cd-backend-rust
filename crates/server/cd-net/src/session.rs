// crates/cd-net/src/session.rs
use crate::error::{NetError, NetResult};
use crate::manager::SharedManager;
use crate::protocol::ServerPacket;
use cd_core::ObjectGuid;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Легкий клонируемый хэндл сессии для передачи в обработчики.
#[derive(Clone)]
pub struct Session {
    pub id: u64,
    manager: SharedManager,
    // Локальный кэш для O(1) проверок авторизации внутри роутера
    guid_cache: Arc<RwLock<Option<ObjectGuid>>>,
}

impl Session {
    pub fn new(id: u64, manager: SharedManager) -> Self {
        Self {
            id,
            manager,
            guid_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Отправляет пакет клиенту напрямую через менеджера
    pub async fn send_packet(&self, packet: ServerPacket) {
        self.manager.send_to_session(self.id, packet).await;
    }

    /// Привязывает GUID к этой сессии
    pub async fn set_authenticated(&self, guid: ObjectGuid) {
        *self.guid_cache.write().await = Some(guid);
        // Делегируем сложную логику (вытеснение, флаг буферов) менеджеру
        self.manager.authenticate_agent(self.id, guid).await;
    }

    pub async fn get_guid(&self) -> Option<ObjectGuid> {
        *self.guid_cache.read().await
    }

    /// Хелпер для middleware проверки авторизации
    pub async fn require_guid(&self) -> NetResult<ObjectGuid> {
        self.get_guid().await.ok_or(NetError::Unauthorized)
    }
}
