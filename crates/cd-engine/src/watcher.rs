use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// Запускает file watcher в отдельном потоке.
/// При изменении .cdb файла вызывает on_change.
pub fn spawn_depot_watcher(
    path: PathBuf,
    on_change: impl Fn(&Path) + Send + 'static,
) -> Result<RecommendedWatcher, notify::Error> {
    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    std::thread::spawn(move || {
        // Debounce: ждём 200ms после последнего события перед вызовом callback
        loop {
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(event)) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        // Дренируем накопившиеся события (burst protection)
                        while rx.recv_timeout(Duration::from_millis(50)).is_ok() {}
                        on_change(&path);
                    }
                }
                Ok(Err(e)) => tracing::warn!("Watcher error: {}", e),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
    });

    Ok(watcher)
}