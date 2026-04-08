use axum::extract::State;
use axum::response::Json;

use metaygn_core::context::LoopContext;
use metaygn_core::topology::TopologyPlanner;
use metaygn_shared::budget_tracker::COST_PER_TOKEN_USD;
use metaygn_shared::protocol::{HookInput, HookOutput};
use metaygn_shared::state::TaskType;

use crate::app_state::AppState;

use super::{append_latency, append_session_budget, format_readable, record_replay};

/// POST /hooks/user-prompt-submit
///
/// 1. Run ControlLoop stages 1-6 on the user's prompt
/// 2. Record a fatigue signal for the prompt
/// 3. Return risk, strategy, and budget recommendation
pub(crate) async fn user_prompt_submit(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let start = std::time::Instant::now();
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event(&session_id, "user_prompt_submit", &payload)
        .await;

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    // Wire fatigue signal: record in both session-local and global profilers.
    let prompt_text = input.prompt.clone().unwrap_or_default();
    let now_utc = chrono::Utc::now();
    {
        if let Ok(mut sess) = session_ctx.lock() {
            sess.fatigue.on_prompt(&prompt_text, now_utc);
        } else {
            tracing::warn!("session mutex poisoned — skipping session fatigue on_prompt");
        }
    }
    {
        if let Ok(mut profiler) = state.fatigue.lock() {
            profiler.on_prompt(&prompt_text, now_utc);
        } else {
            tracing::warn!("fatigue mutex poisoned — skipping global fatigue on_prompt");
        }
    }

    // Estimate and consume tokens: both session-local and global budget.
    {
        let estimated_tokens = (prompt_text.len() / 4) as u64;
        let estimated_cost = estimated_tokens as f64 * COST_PER_TOKEN_USD;
        if let Ok(mut sess) = session_ctx.lock() {
            sess.budget.consume(estimated_tokens, estimated_cost);
        } else {
            tracing::warn!("session mutex poisoned — skipping session budget consume");
        }
        if let Ok(mut budget) = state.budget.lock() {
            budget.consume(estimated_tokens, estimated_cost);
        } else {
            tracing::warn!("budget mutex poisoned — skipping global budget consume");
        }
    }

    // Run ControlLoop stages 1-6 for analysis
    let mut ctx = LoopContext::new(input);
    state.control_loop.run_range(&mut ctx, 0, 6);

    // Persist control loop results and sync tokens_consumed in one lock
    {
        if let Ok(mut sess) = session_ctx.lock() {
            sess.task_type = ctx.task_type;
            sess.risk = ctx.risk;
            sess.strategy = ctx.strategy;
            sess.difficulty = ctx.difficulty;
            sess.competence = ctx.competence;
            sess.tokens_consumed = sess.budget.consumed_tokens();
        } else {
            tracing::warn!("session mutex poisoned — skipping control loop results persist");
        }
    }

    let risk_label = match ctx.risk {
        metaygn_shared::state::RiskLevel::Low => "low",
        metaygn_shared::state::RiskLevel::Medium => "medium",
        metaygn_shared::state::RiskLevel::High => "high",
    };

    // Run TopologyPlanner to determine execution topology
    let task_type = ctx.task_type.unwrap_or(TaskType::Bugfix);
    let plan = TopologyPlanner::plan(ctx.risk, ctx.difficulty, task_type);

    // Store execution plan in session for use by the stop handler
    {
        if let Ok(mut sess) = session_ctx.lock() {
            sess.execution_plan = Some(plan.clone());
        } else {
            tracing::warn!("session mutex poisoned — skipping execution plan persist");
        }
    }

    let strategy_label = format!("{:?}", ctx.strategy).to_lowercase();
    let strategy_kebab = strategy_label.replace('_', "-");
    let task_label = ctx
        .task_type
        .map(|t| format!("{:?}", t).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    let topology_label = format!("{:?}", plan.topology).to_lowercase();

    let readable = format_readable(
        risk_label,
        &strategy_kebab,
        ctx.budget.max_tokens,
        &task_label,
        &topology_label,
    );

    let mut output = HookOutput::context(
        "UserPromptSubmit".to_string(),
        format!("{} | Prompt analysed via control loop", readable),
    );
    append_session_budget(&mut output, &session_ctx);
    append_latency(&mut output, start);
    let resp_json = serde_json::to_string(&output).unwrap_or_default();
    record_replay(
        &state,
        &session_id,
        "UserPromptSubmit",
        &payload,
        &resp_json,
        start,
    )
    .await;

    // System 2: async post-processing (fire-and-forget)
    {
        let state_clone = state.clone();
        let session_clone = session_ctx.clone();
        tokio::spawn(async move {
            crate::postprocess::after_user_prompt_submit(state_clone, session_clone).await;
        });
    }

    Json(output)
}
