// crates/cd-net/src/manager.rs
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

// ─── Внутренние типы ──────────────────────────────────────────────────────────

struct SessionEntry {
    tx: mpsc::Sender<ServerPacket>,
    /// None = сессия анонимна (до аутентификации)
    agent_guid: Option<ObjectGuid>,
}

enum AgentState {
    /// Агент онлайн, пакеты идут напрямую в канал.
    Online { session_id: u64 },
    /// Кратковременный обрыв — пакеты копим, ждём возврата.
    Reconnecting {
        deadline: Instant,
        buffer: VecDeque<ServerPacket>,
    },
}

// ─── Конфиг ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ManagerConfig {
    /// Сколько времени агент может быть оффлайн и всё ещё забрать пакеты.
    pub reconnect_window: Duration,
    /// Максимум буферизованных пакетов (защита от OOM на старом железе).
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

// ─── Менеджер ─────────────────────────────────────────────────────────────────

/// Управляет сессиями и агентами.
///
/// Сессия — эфемерное WebSocket-соединение (session_id).
/// Агент — персистентная игровая сущность (ObjectGuid).
///
/// При разрыве связи агент переходит в `Reconnecting`:
/// пакеты буферизуются и доставляются при переподключении.
/// Если окно истекло — агент считается ушедшим насовсем.
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
        self.sessions.insert(
            session_id,
            SessionEntry {
                tx,
                agent_guid: None,
            },
        );
        tracing::debug!(session_id, "Session opened");
    }

    /// Соединение закрыто (обрыв, таймаут, etc.).
    /// Если агент был привязан — запускаем окно переподключения.
    pub fn remove_session(&self, session_id: u64) {
        if let Some((_, entry)) = self.sessions.remove(&session_id)
            && let Some(guid) = entry.agent_guid
        {
            self.begin_reconnect(guid, session_id);
        }
        tracing::debug!(session_id, "Session closed");
    }

    // ── Агенты ────────────────────────────────────────────────────────────────

    /// Аутентификация: сессия → агент.
    ///
    /// Варианты:
    /// - Новый агент: просто регистрируем.
    /// - Агент был `Reconnecting` в пределах окна: флашим буфер.
    /// - Агент уже `Online` с другой сессии: выбиваем старую (двойной логин).
    pub async fn authenticate_agent(&self, session_id: u64, guid: ObjectGuid) {
        // Привязываем guid к сессии (sync, без await)
        if let Some(mut entry) = self.sessions.get_mut(&session_id) {
            entry.agent_guid = Some(guid);
        }

        // Забираем предыдущее состояние агента. Всё sync — до любого await.
        let prev = self.agents.remove(&guid);
        let (evict_session, buffered) = match prev {
            Some((_, AgentState::Reconnecting { deadline, buffer }))
                if Instant::now() < deadline =>
            {
                tracing::info!(
                    ?guid,
                    session_id,
                    buffered = buffer.len(),
                    "Agent reconnected within window"
                );
                (None, Some(buffer))
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

        // Фиксируем новое состояние
        self.agents.insert(guid, AgentState::Online { session_id });

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
            s.agent_guid = None;
        }
        tracing::info!(?guid, "Agent logged out");
    }

    // ── Отправка ──────────────────────────────────────────────────────────────

    /// Unicast агенту по GUID.
    /// `Reconnecting` → пакет буферизуется.
    /// `Online` → пакет идёт напрямую.
    pub async fn send_to_agent(&self, guid: ObjectGuid, packet: ServerPacket) {
        // Разбираемся с состоянием без удержания RefMut через await.
        let session_id = {
            let Some(mut entry) = self.agents.get_mut(&guid) else {
                tracing::warn!(?guid, "send_to_agent: unknown agent");
                return;
            };
            match entry.value_mut() {
                AgentState::Online { session_id } => *session_id,
                AgentState::Reconnecting { buffer, .. } => {
                    if buffer.len() < self.config.buffer_cap {
                        buffer.push_back(packet);
                    } else {
                        tracing::warn!(?guid, "Buffer full, packet dropped");
                    }
                    return; // пакет либо буферизован, либо дропнут
                }
            }
        }; // ← RefMut упал здесь, можно await

        self.send_to_session(session_id, packet).await;
    }

    /// Multicast: одному пакету — несколько получателей.
    pub async fn send_to_agents(&self, guids: &[ObjectGuid], packet: ServerPacket) {
        for &guid in guids {
            self.send_to_agent(guid, packet.clone()).await;
        }
    }

    /// Низкоуровневая отправка по session_id.
    /// При мёртвом канале — сессия чистится автоматически.
    pub async fn send_to_session(&self, session_id: u64, packet: ServerPacket) {
        let tx = self.sessions.get(&session_id).map(|e| e.tx.clone());
        if let Some(tx) = tx
            && tx.send(packet).await.is_err()
        {
            tracing::warn!(session_id, "Dead channel — removing session");
            self.remove_session(session_id);
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

    fn begin_reconnect(&self, guid: ObjectGuid, session_id: u64) {
        let is_owner = self
            .agents
            .get(&guid)
            .map(|e| matches!(e.value(), AgentState::Online { session_id: sid } if *sid == session_id))
            .unwrap_or(false);

        if is_owner {
            let deadline = Instant::now() + self.config.reconnect_window;
            self.agents.insert(
                guid,
                AgentState::Reconnecting {
                    deadline,
                    buffer: VecDeque::new(),
                },
            );
            tracing::info!(
                ?guid, session_id,
                window = ?self.config.reconnect_window,
                "Reconnect window started"
            );
        }
    }

    fn cleanup_expired(&self) {
        let now = Instant::now();
        let expired: Vec<ObjectGuid> = self
            .agents
            .iter()
            .filter_map(|e| match e.value() {
                AgentState::Reconnecting { deadline, .. } if now > *deadline => Some(*e.key()),
                _ => None,
            })
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
