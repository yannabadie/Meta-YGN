//! Lightweight Prometheus-compatible metrics endpoint.
//! Feature-gated behind `otel` but uses no external metrics crate ---
//! just formats internal counters as Prometheus text.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use crate::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/metrics", get(prometheus_metrics))
}

async fn prometheus_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let mut output = String::new();

    // Sessions
    let session_count = state.sessions.count();
    output.push_str(&format!(
        "# HELP metaygn_active_sessions Current active sessions\n\
         # TYPE metaygn_active_sessions gauge\n\
         metaygn_active_sessions {}\n\n",
        session_count
    ));

    // Memory events
    if let Ok(event_count) = state.memory.event_count().await {
        output.push_str(&format!(
            "# HELP metaygn_events_total Total events logged\n\
             # TYPE metaygn_events_total counter\n\
             metaygn_events_total {}\n\n",
            event_count
        ));
    }

    // Graph nodes
    if let Ok(node_count) = state.graph.node_count().await {
        output.push_str(&format!(
            "# HELP metaygn_graph_nodes_total Total graph memory nodes\n\
             # TYPE metaygn_graph_nodes_total counter\n\
             metaygn_graph_nodes_total {}\n\n",
            node_count
        ));
    }

    // Fatigue
    {
        let profiler = state.fatigue.lock().expect("fatigue mutex poisoned");
        let report = profiler.assess();
        output.push_str(&format!(
            "# HELP metaygn_fatigue_score Current fatigue score\n\
             # TYPE metaygn_fatigue_score gauge\n\
             metaygn_fatigue_score {:.4}\n\n",
            report.score
        ));
    }

    // Budget
    {
        let budget = state.budget.lock().expect("budget mutex poisoned");
        output.push_str(&format!(
            "# HELP metaygn_tokens_consumed_total Total tokens consumed globally\n\
             # TYPE metaygn_tokens_consumed_total counter\n\
             metaygn_tokens_consumed_total {}\n\n",
            budget.consumed_tokens()
        ));
    }

    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        output,
    )
}
