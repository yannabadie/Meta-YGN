//! Multi-objective fitness scoring for heuristic evolution.
//!
//! Fitness is computed from three objectives (AlphaEvolve-inspired):
//! - **Verification success rate** — how often verifications passed (weight: 0.5)
//! - **Token efficiency** — inverse of token waste (weight: 0.3)
//! - **Latency score** — inverse of time spent (weight: 0.2)

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────
// FitnessScore
// ──────────────────────────────────────────────────────────────

/// Multi-objective fitness score for a heuristic version.
///
/// Each dimension is normalised to `[0.0, 1.0]`. The `composite` field is a
/// weighted average: `0.5 * success + 0.3 * token_eff + 0.2 * latency`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessScore {
    /// How often verifications passed (0.0–1.0).
    pub verification_success_rate: f64,
    /// Inverse of token waste (0.0–1.0).
    pub token_efficiency: f64,
    /// Inverse of time spent (0.0–1.0).
    pub latency_score: f64,
    /// Weighted average of the three objectives.
    pub composite: f64,
}

impl FitnessScore {
    /// Compute a fitness score from the three normalised objectives.
    pub fn compute(success_rate: f64, token_efficiency: f64, latency_score: f64) -> Self {
        let composite = success_rate * 0.5 + token_efficiency * 0.3 + latency_score * 0.2;
        Self {
            verification_success_rate: success_rate,
            token_efficiency,
            latency_score,
            composite,
        }
    }

    /// A zero/default fitness (used for newly-created heuristics with no data).
    pub fn zero() -> Self {
        Self {
            verification_success_rate: 0.0,
            token_efficiency: 0.0,
            latency_score: 0.0,
            composite: 0.0,
        }
    }
}

impl Default for FitnessScore {
    fn default() -> Self {
        Self::zero()
    }
}

// ──────────────────────────────────────────────────────────────
// SessionOutcome
// ──────────────────────────────────────────────────────────────

/// Outcome of a session/task, used for statistical learning.
///
/// Each outcome records what strategy was used for a given risk level and
/// whether the session succeeded, along with resource-consumption metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutcome {
    /// Unique session identifier.
    pub session_id: String,
    /// The type of task (e.g. "bugfix", "feature", "security").
    pub task_type: String,
    /// Risk level that was classified (e.g. "low", "medium", "high").
    pub risk_level: String,
    /// Strategy that was selected (e.g. "single", "vertical", "horizontal").
    pub strategy_used: String,
    /// Whether the session completed successfully.
    pub success: bool,
    /// Total tokens consumed during the session.
    pub tokens_consumed: u64,
    /// Wall-clock duration of the session in milliseconds.
    pub duration_ms: u64,
    /// Number of errors encountered during the session.
    pub errors_encountered: u32,
}

// ──────────────────────────────────────────────────────────────
// Unit tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_weighted_average() {
        let f = FitnessScore::compute(1.0, 1.0, 1.0);
        assert!((f.composite - 1.0).abs() < 1e-9);

        let f2 = FitnessScore::compute(0.0, 0.0, 0.0);
        assert!((f2.composite).abs() < 1e-9);
    }

    #[test]
    fn zero_fitness() {
        let f = FitnessScore::zero();
        assert_eq!(f.composite, 0.0);
        assert_eq!(f.verification_success_rate, 0.0);
    }

    #[test]
    fn composite_weights() {
        // success=0.8, tokens=0.6, latency=0.4
        // composite = 0.8*0.5 + 0.6*0.3 + 0.4*0.2 = 0.4 + 0.18 + 0.08 = 0.66
        let f = FitnessScore::compute(0.8, 0.6, 0.4);
        assert!((f.composite - 0.66).abs() < 1e-9);
    }
}
