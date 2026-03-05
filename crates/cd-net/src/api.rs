use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// Текущий снапшот состояния движка для внешних инструментов.
/// Обновляется engine thread после каждого тика.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ApiState {
    pub tick: u64,
    pub entity_count: u32,
    pub entities: Vec<ApiEntity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiEntity {
    pub guid: String,
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: String,
}

/// Arc<Mutex<...>> — shared между engine thread (запись) и HTTP handler (чтение).
pub type SharedApiState = Arc<Mutex<ApiState>>;

/// GET /api/state — отдаёт текущий снапшот движка.
pub async fn handler_get_state(
    State(state): State<SharedApiState>,
) -> Json<ApiState> {
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(s.clone())
}