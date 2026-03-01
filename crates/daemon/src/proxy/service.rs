use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};

use super::pruner::{ContextPruner, Message};
use crate::app_state::AppState;

/// Request body: Anthropic messages format.
#[derive(Debug, Deserialize)]
pub struct PruneRequest {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub max_tokens: u64,
}

/// Response: potentially pruned messages + metadata.
#[derive(Debug, Serialize)]
pub struct PruneResponse {
    pub messages: Vec<Message>,
    pub pruned: bool,
    pub tokens_removed: usize,
    pub recovery_injected: bool,
    pub reason: Option<String>,
}

/// POST /proxy/anthropic
///
/// Receives an Anthropic messages payload, analyses it for reasoning lock-in,
/// and returns either the original or a pruned version with a recovery
/// injection.  The client then forwards the (possibly modified) payload to
/// the real Anthropic API.
pub async fn prune_messages(
    State(state): State<AppState>,
    Json(req): Json<PruneRequest>,
) -> Json<PruneResponse> {
    let pruner = ContextPruner::with_defaults();
    let analysis = pruner.analyze(&req.messages);

    if analysis.should_prune {
        let pruned = pruner.prune(&req.messages);
        let tokens_removed = estimate_tokens(&req.messages)
            .saturating_sub(estimate_tokens(&pruned));
        let level = state.plasticity.lock().unwrap().amplification_level();
        let reason = pruner.amplified_recovery(
            &format!("{} consecutive errors detected", analysis.consecutive_errors),
            level,
        );

        // Record the recovery injection.
        state.plasticity.lock().unwrap().record_recovery_injected();

        Json(PruneResponse {
            messages: pruned,
            pruned: true,
            tokens_removed,
            recovery_injected: true,
            reason: Some(reason),
        })
    } else {
        Json(PruneResponse {
            messages: req.messages,
            pruned: false,
            tokens_removed: 0,
            recovery_injected: false,
            reason: None,
        })
    }
}

/// Rough token estimate: ~4 characters per token.
fn estimate_tokens(messages: &[Message]) -> usize {
    messages.iter().map(|m| m.content.len() / 4).sum()
}
