use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use metaygn_core::heuristics::entropy::EntropyTracker;
use metaygn_core::heuristics::mop::MopDetector;
use metaygn_core::sequence_monitor::SequenceMonitor;
use metaygn_core::topology::ExecutionPlan;
use metaygn_shared::budget_tracker::SessionBudget;
use metaygn_shared::state::*;

use crate::profiler::fatigue::FatigueProfiler;
use crate::profiler::plasticity::PlasticityTracker;

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
    /// Graph node ID of the Task node for this session (for edge creation).
    pub task_node_id: Option<String>,
    /// Graph node ID of the last Evidence node (for edge chaining).
    pub last_evidence_node_id: Option<String>,
    /// Session-local fatigue profiler (avoids cross-session bleed).
    pub fatigue: FatigueProfiler,
    /// Session-local plasticity tracker (avoids cross-session bleed).
    pub plasticity: PlasticityTracker,
    /// Session-local budget tracker (avoids cross-session bleed).
    pub budget: SessionBudget,
    /// Session-local sequence monitor for multi-action pattern detection.
    pub sequence_monitor: SequenceMonitor,
    /// Session-local MOP detector for behavioral meltdown detection.
    pub mop_detector: MopDetector,
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
            task_node_id: None,
            last_evidence_node_id: None,
            fatigue: FatigueProfiler::with_defaults(),
            plasticity: PlasticityTracker::new(),
            budget: SessionBudget::new(100_000, 1.00),
            sequence_monitor: SequenceMonitor::new(),
            mop_detector: MopDetector::new(),
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
        let mut map = self.sessions.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("session store mutex poisoned — recovering");
            poisoned.into_inner()
        });
        map.entry(session_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(SessionContext::new(session_id.to_string()))))
            .clone()
    }

    /// Get a session context without creating one.
    /// Returns `None` if the session does not exist.
    pub fn get(&self, session_id: &str) -> Option<Arc<Mutex<SessionContext>>> {
        let map = self.sessions.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("session store mutex poisoned — recovering");
            poisoned.into_inner()
        });
        map.get(session_id).cloned()
    }

    /// Remove a session (called at session end).
    pub fn remove(&self, session_id: &str) -> Option<Arc<Mutex<SessionContext>>> {
        let mut map = self.sessions.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("session store mutex poisoned — recovering");
            poisoned.into_inner()
        });
        map.remove(session_id)
    }

    /// Number of active sessions.
    pub fn count(&self) -> usize {
        let map = self.sessions.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("session store mutex poisoned — recovering");
            poisoned.into_inner()
        });
        map.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
