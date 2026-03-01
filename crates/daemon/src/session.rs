use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use metaygn_core::heuristics::entropy::EntropyTracker;
use metaygn_core::topology::ExecutionPlan;
use metaygn_shared::state::*;

/// Accumulated state for a single Claude Code session, persisted across hooks.
pub struct SessionContext {
    pub session_id: String,
    pub created_at: Instant,
    pub task_type: Option<TaskType>,
    pub risk: RiskLevel,
    pub strategy: Strategy,
    pub difficulty: f32,
    pub competence: f32,
    pub entropy_tracker: EntropyTracker,
    pub metacog_vector: MetacognitiveVector,
    pub verification_results: Vec<String>,
    pub lessons: Vec<String>,
    pub execution_plan: Option<ExecutionPlan>,
    pub tool_calls: u32,
    pub errors: u32,
    pub success_count: u32,
    pub tokens_consumed: u64,
}

impl SessionContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            created_at: Instant::now(),
            task_type: None,
            risk: RiskLevel::Low,
            strategy: Strategy::StepByStep,
            difficulty: 0.5,
            competence: 0.7,
            entropy_tracker: EntropyTracker::new(20),
            metacog_vector: MetacognitiveVector {
                confidence: 0.5,
                coherence: 0.5,
                grounding: 0.5,
                complexity: 0.5,
                progress: 0.0,
            },
            verification_results: Vec::new(),
            lessons: Vec::new(),
            execution_plan: None,
            tool_calls: 0,
            errors: 0,
            success_count: 0,
            tokens_consumed: 0,
        }
    }
}

/// Thread-safe store of active sessions.
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Arc<Mutex<SessionContext>>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Get or create a session context for the given session ID.
    pub fn get_or_create(&self, session_id: &str) -> Arc<Mutex<SessionContext>> {
        let mut map = self.sessions.lock().expect("session store mutex poisoned");
        map.entry(session_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(SessionContext::new(session_id.to_string()))))
            .clone()
    }

    /// Remove a session (called at session end).
    pub fn remove(&self, session_id: &str) -> Option<Arc<Mutex<SessionContext>>> {
        let mut map = self.sessions.lock().expect("session store mutex poisoned");
        map.remove(session_id)
    }

    /// Number of active sessions.
    pub fn count(&self) -> usize {
        let map = self.sessions.lock().expect("session store mutex poisoned");
        map.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
