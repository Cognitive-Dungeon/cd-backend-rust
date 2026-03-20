use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

/// Снапшот состояния движка для внешних инструментов (REST API, отладка).
/// Обновляется engine thread после каждого тика.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ApiState {
    pub tick: u64,
    pub entity_count: u32,
    pub entities: Vec<ApiEntity>,
}

/// Плоское представление сущности для REST API.
#[derive(Debug, Clone, Serialize)]
pub struct ApiEntity {
    pub guid: String,
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: String,
}

/// Разделяемое состояние между engine thread (запись) и HTTP-хэндлером (чтение).
pub type SharedApiState = Arc<Mutex<ApiState>>;

/// Callback для горячей перезагрузки игровых данных.
/// Движок регистрирует его при старте, HTTP-хэндлер вызывает по запросу.
pub type ReloadCallback = Arc<TokioMutex<Box<dyn Fn() + Send + 'static>>>;

pub mod handlers;
