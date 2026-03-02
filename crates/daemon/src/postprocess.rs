//! Async post-processing (System 2): background tasks spawned AFTER hook
//! handlers return their HTTP response.  These are fire-and-forget operations
//! that populate the memory graph and update session-level trackers without
//! blocking the fast path.

use std::sync::{Arc, Mutex};

use metaygn_core::heuristics::fitness::SessionOutcome;
use metaygn_memory::graph::{EdgeType, MemoryEdge, MemoryNode, NodeType, Scope};

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

    let task_id = format!("task-{}", uuid::Uuid::new_v4());
    let content = format!(
        "task_type={}, risk={}, strategy={}",
        task_type, risk, strategy
    );
    let node = MemoryNode {
        id: task_id.clone(),
        node_type: NodeType::Task,
        scope: Scope::Session,
        label: format!("Task: {} (risk: {})", task_type, risk),
        embedding: state.embedding.embed(&content).ok(),
        content,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    };

    if let Err(e) = state.graph.insert_node(&node).await {
        tracing::warn!("failed to insert Task node into graph: {e}");
    }

    // Store task node ID in session for edge creation in subsequent hooks
    session.lock().unwrap().task_node_id = Some(task_id);
}

/// After `post_tool_use`: update the entropy tracker with the latest
/// confidence / success signal, then insert an Evidence node into the graph.
/// Tier 2: run async forge verification for Python files written by the agent.
pub async fn after_post_tool_use(
    state: AppState,
    session: Arc<Mutex<SessionContext>>,
    tool_name: String,
    was_error: bool,
    file_path: Option<String>,
) {
    // 1. Update entropy tracker in session
    {
        let mut sess = session.lock().unwrap();
        let confidence = sess.metacog_vector.confidence;
        sess.entropy_tracker.record(confidence, !was_error);
    }

    // 2. Insert Evidence node into graph + edge from Task
    let evidence_id = format!("evidence-{}", uuid::Uuid::new_v4());
    let content = format!("tool={}, error={}", tool_name, was_error);
    let node = MemoryNode {
        id: evidence_id.clone(),
        node_type: NodeType::Evidence,
        scope: Scope::Session,
        label: format!(
            "Tool: {} ({})",
            tool_name,
            if was_error { "error" } else { "success" }
        ),
        embedding: state.embedding.embed(&content).ok(),
        content,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    };

    if let Err(e) = state.graph.insert_node(&node).await {
        tracing::warn!("failed to insert Evidence node into graph: {e}");
    }

    // Create edge: Task → Evidence (DependsOn)
    let task_node_id = session.lock().unwrap().task_node_id.clone();
    if let Some(task_id) = task_node_id {
        let edge = MemoryEdge {
            source_id: task_id,
            target_id: evidence_id.clone(),
            edge_type: EdgeType::Produces,
            weight: 1.0,
            metadata: None,
        };
        let _ = state.graph.insert_edge(&edge).await;
    }
    // Store for chaining
    session.lock().unwrap().last_evidence_node_id = Some(evidence_id);

    // 3. Tier 2: async forge verification for Python files written to disk
    let is_python_file = file_path
        .as_ref()
        .map(|p| p.ends_with(".py"))
        .unwrap_or(false);
    if (tool_name == "Write" || tool_name == "Edit") && is_python_file {
        // Read actual file content from disk (not tool_response which is the hook message)
        if let Some(ref path) = file_path {
            match tokio::fs::read_to_string(path).await {
                Ok(content) if !content.is_empty() && content.len() < 10_000 => {
                    let mut tmp_forge = crate::forge::ForgeEngine::new(state.sandbox.clone());
                    let params = std::collections::HashMap::new();
                    if let Ok(spec) = tmp_forge.generate("syntax-checker", &params) {
                        match tmp_forge.execute(&spec, &content).await {
                            Ok(result) => {
                                if !result.success
                                    || result.stdout.contains("\"valid\": false")
                                    || result.stdout.contains("\"valid\":false")
                                {
                                    let mut sess = session.lock().unwrap();
                                    sess.verification_results.push(format!(
                                        "syntax_check_failed: Python syntax error in {}",
                                        path
                                    ));
                                    tracing::warn!(file = %path, "Tier 2 forge: Python syntax error");
                                }
                            }
                            Err(e) => tracing::debug!("Tier 2 forge execution failed: {e}"),
                        }
                    }
                }
                Ok(_) => {} // empty or too large, skip
                Err(e) => tracing::debug!(file = ?path, "Tier 2: could not read file: {e}"),
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

    // 1. Insert Decision node + edge from last Evidence
    let decision_id = format!("decision-{}", uuid::Uuid::new_v4());
    let decision_node = MemoryNode {
        id: decision_id.clone(),
        node_type: NodeType::Decision,
        scope: Scope::Session,
        label: format!("Decision: {}", decision),
        embedding: state.embedding.embed(&decision).ok(),
        content: decision.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    };
    if let Err(e) = state.graph.insert_node(&decision_node).await {
        tracing::warn!("failed to insert Decision node: {e}");
    }

    // Create edge: last Evidence → Decision (Verifies)
    let last_evidence_id = session.lock().unwrap().last_evidence_node_id.clone();
    if let Some(evidence_id) = last_evidence_id {
        let edge = MemoryEdge {
            source_id: evidence_id,
            target_id: decision_id.clone(),
            edge_type: EdgeType::Verifies,
            weight: 1.0,
            metadata: None,
        };
        let _ = state.graph.insert_edge(&edge).await;
    }

    // 2. Insert Lesson nodes (max 5 to avoid flooding)
    for (i, lesson) in lessons.iter().take(5).enumerate() {
        let lesson_node = MemoryNode {
            id: format!("lesson-{}-{}", session_id, i),
            node_type: NodeType::Lesson,
            scope: Scope::Project,
            label: lesson.clone(),
            embedding: state.embedding.embed(lesson).ok(),
            content: lesson.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            access_count: 0,
        };
        if let Err(e) = state.graph.insert_node(&lesson_node).await {
            tracing::warn!("failed to insert Lesson node: {e}");
        }
    }

    // 3. Record session outcome for heuristic evolution
    let (
        task_type,
        risk,
        strategy,
        errors,
        _success_count,
        duration_ms,
        tokens_consumed,
        tool_calls,
    ) = {
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
            sess.tool_calls,
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
        tool_calls = tool_calls,
        "session outcome recorded for heuristic evolution"
    );
}
