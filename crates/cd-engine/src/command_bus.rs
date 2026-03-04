use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use crate::input::InputCmd;

pub type CommandSeq = u64;

/// Команда с гарантией порядка
#[derive(Debug, Clone)]
pub struct StampedCommand {
    /// Монотонный глобальный счётчик — основа для сортировки
    pub seq: CommandSeq,
    pub payload: InputCmd,
}

/// Клонируемый handle для отправки команд (для сети и агентов)
#[derive(Clone)]
pub struct CommandSender {
    tx: mpsc::Sender<StampedCommand>,
    seq_counter: Arc<AtomicU64>,
}

impl CommandSender {
    pub async fn send(&self, cmd: InputCmd) -> Result<(), mpsc::error::SendError<StampedCommand>> {
        let seq = self.seq_counter.fetch_add(1, Ordering::Relaxed);
        self.tx.send(StampedCommand { seq, payload: cmd }).await
    }
}

/// Шина команд — живёт рядом с Engine, на его потоке
pub struct CommandBus {
    rx: mpsc::Receiver<StampedCommand>,
    seq_counter: Arc<AtomicU64>,
}

impl CommandBus {
    pub fn new(capacity: usize) -> (Self, CommandSender) {
        let (tx, rx) = mpsc::channel(capacity);
        let seq_counter = Arc::new(AtomicU64::new(0));

        let sender = CommandSender {
            tx,
            seq_counter: Arc::clone(&seq_counter),
        };

        (Self { rx, seq_counter }, sender)
    }

    /// Дренирует все накопленные команды, сортирует по seq.
    /// Вызывается ровно один раз в начале каждого тика.
    pub fn drain_sorted(&mut self) -> Vec<StampedCommand> {
        let mut commands = Vec::new();
        while let Ok(cmd) = self.rx.try_recv() {
            commands.push(cmd);
        }
        // Сортировка гарантирует одинаковый порядок при replay
        commands.sort_unstable_by_key(|c| c.seq);
        commands
    }
}