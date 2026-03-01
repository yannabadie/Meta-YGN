use axum::response::Json;
use axum::routing::post;
use axum::{Extension, Router};
use serde_json::{Value, json};
use tokio::sync::watch;

use crate::app_state::AppState;

/// POST /admin/shutdown — initiate graceful daemon shutdown.
pub async fn shutdown(Extension(tx): Extension<watch::Sender<bool>>) -> Json<Value> {
    let _ = tx.send(true);
    Json(json!({"ok": true, "message": "Shutdown initiated"}))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/admin/shutdown", post(shutdown))
}
