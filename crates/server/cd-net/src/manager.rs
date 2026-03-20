use crate::protocol::ServerPacket;
use cd_core::ObjectGuid;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub type SharedManager = Arc<ConnectionManager>;

const DEFAULT_RECONNECT_WINDOW: Duration = Duration::from_secs(30);
const DEFAULT_BUFFER_CAP: usize = 128;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(10);

// ─── Конфиг ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ManagerConfig {
    /// Сколько времени агент может быть оффлайн и всё ещё забрать пакеты.
    pub reconnect_window: Duration,
    /// Максимум буферизованных пакетов (защита от OOM).
    pub buffer_cap: usize,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            reconnect_window: DEFAULT_RECONNECT_WINDOW,
            buffer_cap: DEFAULT_BUFFER_CAP,
        }
    }
}

// ─── Сессия ───────────────────────────────────────────────────────────────────

/// Одно активное WebSocket-соединение.
/// Существует только пока провод жив.
struct SessionEntry {
    /// Канал в сокет этого клиента
    tx: mpsc::Sender<ServerPacket>,
    /// None = клиент ещё не прошёл аутентификацию
    agent_guid: Option<ObjectGuid>,
}

impl SessionEntry {
    fn new(tx: mpsc::Sender<ServerPacket>) -> Self {
        Self {
            tx,
            agent_guid: None,
        }
    }

    fn bind_agent(&mut self, guid: ObjectGuid) {
        self.agent_guid = Some(guid);
    }

    fn unbind_agent(&mut self) {
        self.agent_guid = None;
    }
}

// ─── Агент ────────────────────────────────────────────────────────────────────

/// Состояние игровой сущности с точки зрения сетевого слоя.
/// Агент существует независимо от конкретного WebSocket-соединения.
enum AgentState {
    /// Агент онлайн — пакеты идут напрямую в сессию.
    Online { session_id: u64 },
    /// Кратковременный обрыв — пакеты копим, ждём возврата.
    Reconnecting {
        deadline: Instant,
        buffer: VecDeque<ServerPacket>,
    },
}

impl AgentState {
    fn online(session_id: u64) -> Self {
        Self::Online { session_id }
    }

    fn reconnecting(window: Duration) -> Self {
        Self::Reconnecting {
            deadline: Instant::now() + window,
            buffer: VecDeque::new(),
        }
    }

    fn is_within_reconnect_window(&self) -> bool {
        matches!(self, Self::Reconnecting { deadline, .. } if Instant::now() < *deadline)
    }

    fn is_expired(&self) -> bool {
        matches!(self, Self::Reconnecting { deadline, .. } if Instant::now() > *deadline)
    }

    /// Если агент в режиме переподключения — буферизуем пакет.
    /// Возвращает true если пакет был буферизован (и не нужно слать напрямую).
    fn try_buffer(&mut self, packet: ServerPacket, cap: usize) -> bool {
        match self {
            Self::Reconnecting { buffer, .. } => {
                if buffer.len() < cap {
                    buffer.push_back(packet);
                } else {
                    tracing::warn!("Reconnect buffer full, packet dropped");
                }
                true
            }
            Self::Online { .. } => false,
        }
    }

    /// Забирает накопленный буфер при восстановлении соединения.
    fn drain_buffer(&mut self) -> VecDeque<ServerPacket> {
        match self {
            Self::Reconnecting { buffer, .. } => std::mem::take(buffer),
            Self::Online { .. } => VecDeque::new(),
        }
    }

    fn session_id(&self) -> Option<u64> {
        match self {
            Self::Online { session_id } => Some(*session_id),
            Self::Reconnecting { .. } => None,
        }
    }
}

// ─── Менеджер ─────────────────────────────────────────────────────────────────

/// Управляет сессиями (WebSocket-соединениями) и агентами (игровыми сущностями).
///
/// Ключевое разделение:
/// - **Сессия** — эфемерна, живёт пока жив провод (session_id: u64)
/// - **Агент** — персистентен, переживает обрывы связи (ObjectGuid)
///
/// При разрыве агент переходит в `Reconnecting`: пакеты буферизуются
/// и доставляются при переподключении в пределах `reconnect_window`.
pub struct ConnectionManager {
    sessions: DashMap<u64, SessionEntry>,
    agents: DashMap<ObjectGuid, AgentState>,
    config: ManagerConfig,
}

impl ConnectionManager {
    pub fn new() -> SharedManager {
        Self::with_config(ManagerConfig::default())
    }

    pub fn with_config(config: ManagerConfig) -> SharedManager {
        let manager = Arc::new(Self {
            sessions: DashMap::new(),
            agents: DashMap::new(),
            config,
        });
        manager.clone().spawn_cleanup();
        manager
    }

    pub fn config(&self) -> &ManagerConfig {
        &self.config
    }

    // ── Сессии ────────────────────────────────────────────────────────────────

    /// Новое WebSocket-соединение. Агент пока не известен.
    pub fn add_session(&self, session_id: u64, tx: mpsc::Sender<ServerPacket>) {
        self.sessions.insert(session_id, SessionEntry::new(tx));
        tracing::debug!(session_id, "Session opened");
    }

    /// Соединение закрыто — если к сессии был привязан агент,
    /// запускаем окно переподключения.
    pub fn remove_session(&self, session_id: u64) {
        if let Some((_, entry)) = self.sessions.remove(&session_id)
            && let Some(guid) = entry.agent_guid
        {
            self.begin_reconnect_window(guid, session_id);
        }
        tracing::debug!(session_id, "Session closed");
    }

    // ── Агенты ────────────────────────────────────────────────────────────────

    /// Аутентификация: привязываем сессию к агенту.
    ///
    /// Три сценария:
    /// - **Новый агент** — просто регистрируем
    /// - **Агент возвращается** (был `Reconnecting`) — флашим буфер
    /// - **Двойной логин** (агент уже `Online`) — выбиваем старую сессию
    pub async fn authenticate_agent(&self, session_id: u64, guid: ObjectGuid) {
        // Привязываем guid к сессии (sync, без await)
        if let Some(mut entry) = self.sessions.get_mut(&session_id) {
            entry.bind_agent(guid);
        }

        // Забираем предыдущее состояние агента. Всё sync — до любого await.
        let prev = self.agents.remove(&guid);
        let (evict_session, buffered) = match prev {
            Some((_, mut state)) if state.is_within_reconnect_window() => {
                tracing::info!(?guid, session_id, "Agent reconnected within window");
                (None, Some(state.drain_buffer()))
            }
            Some((
                _,
                AgentState::Online {
                    session_id: old_sid,
                },
            )) if old_sid != session_id => {
                tracing::warn!(
                    ?guid,
                    old = old_sid,
                    new = session_id,
                    "Double login — evicting old session"
                );
                (Some(old_sid), None)
            }
            _ => (None, None),
        };

        self.agents.insert(guid, AgentState::online(session_id));

        // Убиваем вытесненную сессию
        if let Some(old_sid) = evict_session {
            self.sessions.remove(&old_sid);
        }

        // Доставляем накопленные пакеты
        if let Some(packets) = buffered {
            for packet in packets {
                self.send_to_session(session_id, packet).await;
            }
        }
    }

    /// Явный логаут — окно переподключения не открывается.
    pub fn logout_agent(&self, guid: ObjectGuid) {
        if let Some((_, AgentState::Online { session_id })) = self.agents.remove(&guid)
            && let Some(mut s) = self.sessions.get_mut(&session_id)
        {
            s.unbind_agent();
        }
        tracing::info!(?guid, "Agent logged out");
    }

    // ── Отправка ──────────────────────────────────────────────────────────────

    /// Unicast агенту по GUID.
    /// Если агент переподключается — пакет буферизуется.
    pub async fn send_to_agent(&self, guid: ObjectGuid, packet: ServerPacket) {
        let session_id = {
            let Some(mut entry) = self.agents.get_mut(&guid) else {
                tracing::warn!(?guid, "send_to_agent: unknown agent");
                return;
            };
            if entry.try_buffer(packet.clone(), self.config.buffer_cap) {
                return;
            }
            entry.session_id()
        };

        if let Some(id) = session_id {
            self.send_to_session(id, packet).await;
        }
    }

    /// Multicast: один пакет — несколько получателей.
    pub async fn send_to_agents(&self, guids: &[ObjectGuid], packet: ServerPacket) {
        for &guid in guids {
            self.send_to_agent(guid, packet.clone()).await;
        }
    }

    /// Broadcast всем подключённым сессиям.
    pub async fn broadcast(&self, packet: ServerPacket) {
        // Снимаем снапшот без удержания итератора через await
        let senders: Vec<(u64, mpsc::Sender<ServerPacket>)> = self
            .sessions
            .iter()
            .map(|e| (*e.key(), e.tx.clone()))
            .collect();

        for (session_id, tx) in senders {
            if tx.send(packet.clone()).await.is_err() {
                self.remove_session(session_id);
            }
        }
    }

    /// Низкоуровневая отправка по session_id.
    /// При мёртвом канале сессия чистится автоматически.
    pub async fn send_to_session(&self, session_id: u64, packet: ServerPacket) {
        let tx = self.sessions.get(&session_id).map(|e| e.tx.clone());
        if let Some(tx) = tx
            && tx.send(packet).await.is_err()
        {
            tracing::warn!(session_id, "Dead channel — removing session");
            self.remove_session(session_id);
        }
    }

    // ── Статистика ────────────────────────────────────────────────────────────

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn online_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|e| matches!(e.value(), AgentState::Online { .. }))
            .count()
    }

    pub fn reconnecting_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|e| matches!(e.value(), AgentState::Reconnecting { .. }))
            .count()
    }

    // ── Внутренние ────────────────────────────────────────────────────────────

    fn begin_reconnect_window(&self, guid: ObjectGuid, session_id: u64) {
        let is_owner = self
            .agents
            .get(&guid)
            .map(|e| e.session_id() == Some(session_id))
            .unwrap_or(false);

        if is_owner {
            self.agents
                .insert(guid, AgentState::reconnecting(self.config.reconnect_window));
            tracing::info!(?guid, session_id, window = ?self.config.reconnect_window, "Reconnect window started");
        }
    }

    fn cleanup_expired(&self) {
        let expired: Vec<ObjectGuid> = self
            .agents
            .iter()
            .filter(|e| e.value().is_expired())
            .map(|e| *e.key())
            .collect();

        for guid in &expired {
            self.agents.remove(guid);
            tracing::info!(?guid, "Reconnect window expired, agent evicted");
        }

        if !expired.is_empty() {
            tracing::debug!(count = expired.len(), "Expired agents cleaned up");
        }
    }

    fn spawn_cleanup(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(CLEANUP_INTERVAL);
            loop {
                ticker.tick().await;
                self.cleanup_expired();
            }
        });
    }
}
