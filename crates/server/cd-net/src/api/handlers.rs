use axum::{Json, extract::State, http::StatusCode};

use super::{ApiState, ReloadCallback, SharedApiState};

/// GET /api/state — текущий снапшот движка.
pub async fn get_state(State(state): State<SharedApiState>) -> Json<ApiState> {
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(s.clone())
}

/// POST /api/reload-data — горячая перезагрузка игровых данных без рестарта.
pub async fn reload_data(State(cb): State<ReloadCallback>) -> StatusCode {
    (cb.lock().await)();
    StatusCode::OK
}
