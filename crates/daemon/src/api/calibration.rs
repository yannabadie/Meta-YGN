use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::get};

use crate::app_state::AppState;

/// GET /calibration — returns Brier score and calibration buckets.
async fn get_calibration(State(state): State<AppState>) -> Json<serde_json::Value> {
    let outcomes = state
        .memory
        .load_recent_outcomes(200)
        .await
        .unwrap_or_default();

    let mut pairs: Vec<(f64, f64)> = Vec::new();
    for outcome in &outcomes {
        // Map risk_level to confidence proxy: Low=0.8, Medium=0.5, High=0.3
        let confidence = match outcome.get("risk_level").and_then(|v| v.as_str()) {
            Some("Low") | Some("low") => 0.8,
            Some("Medium") | Some("medium") => 0.5,
            Some("High") | Some("high") => 0.3,
            _ => 0.5,
        };
        // success is stored as a boolean in the JSON from load_recent_outcomes
        let actual = if outcome
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            1.0
        } else {
            0.0
        };
        pairs.push((confidence, actual));
    }

    // Brier score: BS = (1/N) * sum((confidence - outcome)^2)
    let n = pairs.len() as f64;
    let brier = if n > 0.0 {
        pairs.iter().map(|(c, o)| (c - o).powi(2)).sum::<f64>() / n
    } else {
        0.0
    };

    // Calibration buckets: 0-20%, 20-40%, 40-60%, 60-80%, 80-100%
    let mut buckets: Vec<serde_json::Value> = Vec::new();
    for i in 0..5 {
        let lo = i as f64 * 0.2;
        let hi = lo + 0.2;
        let bucket_pairs: Vec<&(f64, f64)> =
            pairs.iter().filter(|(c, _)| *c >= lo && *c < hi).collect();
        let count = bucket_pairs.len();
        let avg_success = if count > 0 {
            bucket_pairs.iter().map(|(_, o)| o).sum::<f64>() / count as f64
        } else {
            0.0
        };
        buckets.push(serde_json::json!({
            "range": format!("{:.0}-{:.0}%", lo * 100.0, hi * 100.0),
            "count": count,
            "avg_predicted": (lo + hi) / 2.0,
            "avg_actual": avg_success,
        }));
    }

    Json(serde_json::json!({
        "brier_score": brier,
        "sample_count": pairs.len(),
        "buckets": buckets,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/calibration", get(get_calibration))
}
