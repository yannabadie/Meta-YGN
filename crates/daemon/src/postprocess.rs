//! Async post-processing (System 2): background tasks spawned AFTER hook
//! handlers return their HTTP response.  These are fire-and-forget operations
//! that populate the memory graph and update session-level trackers without
//! blocking the fast path.

use std::sync::{Arc, Mutex};

use metaygn_core::heuristics::fitness::SessionOutcome;
use metaygn_memory::graph::{MemoryNode, NodeType, Scope};

use crate::app_state::AppState;
use crate::session::SessionContext;

/// Minimum number of accumulated outcomes before triggering evolution.
const EVOLUTION_THRESHOLD: usize = 5;

/// After `user_prompt_submit`: insert a Task node into the memory graph
/// capturing the classified task type, risk level, and chosen strategy.
pub async fn after_user_prompt_submit(state: AppState, session: Arc<Mutex<SessionContext>>) {
    let (task_type, risk, strategy) = {
        let sess = session.lock().unwrap();
        (
            sess.task_type
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "Unknown".into()),
            format!("{:?}", sess.risk),
            format!("{:?}", sess.strategy),
        )
    };

    let node = MemoryNode {
        id: format!("task-{}", uuid::Uuid::new_v4()),
        node_type: NodeType::Task,
        scope: Scope::Session,
        label: format!("Task: {} (risk: {})", task_type, risk),
        content: format!(
            "task_type={}, risk={}, strategy={}",
            task_type, risk, strategy
        ),
        embedding: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    };

    if let Err(e) = state.graph.insert_node(&node).await {
        tracing::warn!("failed to insert Task node into graph: {e}");
    }
}

/// After `post_tool_use`: update the entropy tracker with the latest
/// confidence / success signal, then insert an Evidence node into the graph.
/// Tier 2: run async forge verification for Python files written by the agent.
pub async fn after_post_tool_use(
    state: AppState,
    session: Arc<Mutex<SessionContext>>,
    tool_name: String,
    was_error: bool,
    tool_response: String,
) {
    // 1. Update entropy tracker in session
    {
        let mut sess = session.lock().unwrap();
        let confidence = sess.metacog_vector.confidence;
        sess.entropy_tracker.record(confidence, !was_error);
    }

    // 2. Insert Evidence node into graph
    let node = MemoryNode {
        id: format!("evidence-{}", uuid::Uuid::new_v4()),
        node_type: NodeType::Evidence,
        scope: Scope::Session,
        label: format!(
            "Tool: {} ({})",
            tool_name,
            if was_error { "error" } else { "success" }
        ),
        content: format!("tool={}, error={}", tool_name, was_error),
        embedding: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    };

    if let Err(e) = state.graph.insert_node(&node).await {
        tracing::warn!("failed to insert Evidence node into graph: {e}");
    }

    // 3. Tier 2: async forge verification for Python files
    if tool_name == "Write" && tool_response.contains(".py") {
        let content = &tool_response;
        if !content.is_empty() && content.len() < 10_000 {
            let mut tmp_forge = crate::forge::ForgeEngine::new(state.sandbox.clone());
            let params = std::collections::HashMap::new();
            if let Ok(spec) = tmp_forge.generate("syntax-checker", &params) {
                match tmp_forge.execute(&spec, content).await {
                    Ok(result) => {
                        if !result.success
                            || result.stdout.contains("\"valid\": false")
                            || result.stdout.contains("\"valid\":false")
                        {
                            let mut sess = session.lock().unwrap();
                            sess.verification_results.push(
                                "syntax_check_failed: Python syntax error detected by forge"
                                    .to_string(),
                            );
                            tracing::warn!("Tier 2 forge: Python syntax error detected");
                        }
                    }
                    Err(e) => tracing::debug!("Tier 2 forge execution failed: {e}"),
                }
            }
        }
    }
}

/// After `stop`: insert a Decision node summarising the session outcome,
/// plus up to 5 Lesson nodes (project-scoped) for cross-session learning.
pub async fn after_stop(
    state: AppState,
    session: Arc<Mutex<SessionContext>>,
    decision: String,
    lessons: Vec<String>,
) {
    let session_id = session.lock().unwrap().session_id.clone();

    // 1. Insert Decision node
    let decision_node = MemoryNode {
        id: format!("decision-{}", uuid::Uuid::new_v4()),
        node_type: NodeType::Decision,
        scope: Scope::Session,
        label: format!("Decision: {}", decision),
        content: decision.clone(),
        embedding: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    };
    if let Err(e) = state.graph.insert_node(&decision_node).await {
        tracing::warn!("failed to insert Decision node: {e}");
    }

    // 2. Insert Lesson nodes (max 5 to avoid flooding)
    for (i, lesson) in lessons.iter().take(5).enumerate() {
        let lesson_node = MemoryNode {
            id: format!("lesson-{}-{}", session_id, i),
            node_type: NodeType::Lesson,
            scope: Scope::Project,
            label: lesson.clone(),
            content: lesson.clone(),
            embedding: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            access_count: 0,
        };
        if let Err(e) = state.graph.insert_node(&lesson_node).await {
            tracing::warn!("failed to insert Lesson node: {e}");
        }
    }

    // 3. Record session outcome for heuristic evolution
    let (task_type, risk, strategy, errors, _success_count, duration_ms, tokens_consumed) = {
        let sess = session.lock().unwrap();
        (
            sess.task_type
                .map(|t| format!("{:?}", t))
                .unwrap_or_default(),
            format!("{:?}", sess.risk),
            format!("{:?}", sess.strategy),
            sess.errors,
            sess.success_count,
            sess.created_at.elapsed().as_millis() as u64,
            sess.tokens_consumed,
        )
    };

    let outcome = SessionOutcome {
        session_id: session_id.clone(),
        task_type: task_type.clone(),
        risk_level: risk.clone(),
        strategy_used: strategy.clone(),
        success: errors == 0,
        tokens_consumed,
        duration_ms,
        errors_encountered: errors,
    };

    // Record outcome and evolve if enough data
    let evolved_best = {
        let mut evolver = state.evolver.lock().expect("evolver mutex poisoned");
        evolver.record_outcome(outcome.clone());
        // Evolve after accumulating enough outcomes
        if evolver.outcomes().len() >= EVOLUTION_THRESHOLD {
            evolver.evaluate_all();
            evolver.best().cloned()
        } else {
            None
        }
    };

    // Persist outcome to SQLite
    let outcome_id = uuid::Uuid::new_v4().to_string();
    let _ = state
        .memory
        .save_outcome(
            &outcome_id,
            &outcome.session_id,
            &outcome.task_type,
            &outcome.risk_level,
            &outcome.strategy_used,
            outcome.success,
            outcome.tokens_consumed,
            outcome.duration_ms,
            outcome.errors_encountered,
        )
        .await;

    // If evolution happened, persist the best heuristic version
    if let Some(best) = evolved_best {
        let _ = state
            .memory
            .save_heuristic(
                &best.id,
                best.generation,
                best.parent_id.as_deref(),
                &serde_json::to_string(&best.fitness).unwrap_or_default(),
                &serde_json::to_string(&best.risk_weights).unwrap_or_default(),
                &serde_json::to_string(&best.strategy_scores).unwrap_or_default(),
                &best.created_at,
            )
            .await;
    }

    tracing::info!(
        session_id = %session_id,
        success = errors == 0,
        "session outcome recorded for heuristic evolution"
    );
}
