use crate::events::EngineEvent;
use tokio::sync::broadcast;

/// Port для телеметрии — движок зависит только от этого trait.
/// В production — NullSink (zero overhead).
/// В debug/tools — BroadcastSink.
pub trait TelemetrySink: Send + Sync + 'static {
    fn emit(&self, event: EngineEvent);
}

// ----- NullSink -----

/// Zero-cost заглушка для production.
pub struct NullSink;

impl TelemetrySink for NullSink {
    #[inline(always)]
    fn emit(&self, _event: EngineEvent) {}
}

// ----- BroadcastSink -----

/// Рассылает события всем подключённым WS-клиентам.
pub struct BroadcastSink {
    tx: broadcast::Sender<EngineEvent>,
}

impl BroadcastSink {
    /// Возвращает sink + sender для передачи в сетевой слой.
    pub fn new(capacity: usize) -> (Self, broadcast::Sender<EngineEvent>) {
        let (tx, _) = broadcast::channel(capacity);
        (Self { tx: tx.clone() }, tx)
    }
}

impl TelemetrySink for BroadcastSink {
    fn emit(&self, event: EngineEvent) {
        // Нет подписчиков — нет аллокаций, просто дроп
        let _ = self.tx.send(event);
    }
}