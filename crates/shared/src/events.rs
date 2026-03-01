use serde::{Deserialize, Serialize};

/// Typed metacognitive events replacing ad-hoc string logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaEvent {
    SessionStarted {
        stack: Vec<String>,
        source: String,
    },
    PromptClassified {
        risk: String,
        strategy: String,
        topology: String,
    },
    ToolGated {
        tool: String,
        decision: String,
        guard: String,
        score: u8,
    },
    ToolCompleted {
        tool: String,
        success: bool,
        duration_ms: u64,
    },
    ToolFailed {
        tool: String,
        error: String,
    },
    RecoveryInjected {
        level: u8,
        reason: String,
    },
    RecoveryOutcome {
        success: bool,
        plasticity_score: f64,
    },
    CompletionVerified {
        verified: bool,
        issues: Vec<String>,
    },
    TestIntegrityWarning {
        file: String,
        issues: Vec<String>,
    },
    BudgetConsumed {
        tokens: u64,
        cost_usd: f64,
        utilization: f64,
    },
    SessionEnded {
        reason: String,
    },
}

impl MetaEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::SessionStarted { .. } => "session_started",
            Self::PromptClassified { .. } => "prompt_classified",
            Self::ToolGated { .. } => "tool_gated",
            Self::ToolCompleted { .. } => "tool_completed",
            Self::ToolFailed { .. } => "tool_failed",
            Self::RecoveryInjected { .. } => "recovery_injected",
            Self::RecoveryOutcome { .. } => "recovery_outcome",
            Self::CompletionVerified { .. } => "completion_verified",
            Self::TestIntegrityWarning { .. } => "test_integrity_warning",
            Self::BudgetConsumed { .. } => "budget_consumed",
            Self::SessionEnded { .. } => "session_ended",
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
