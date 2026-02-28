use serde::{Deserialize, Serialize};

/// Outcome of a recovery prompt injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryOutcome {
    /// The developer's next action suggests the recovery prompt worked.
    Success,
    /// The developer repeated the same mistake or ignored the guidance.
    Failure,
}

/// Tracks whether recovery prompts injected by the context pruner are
/// actually effective.  This is "implicit feedback" -- we infer
/// success/failure from subsequent hook events without asking the developer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticityTracker {
    total_injections: u32,
    successes: u32,
    failures: u32,
    consecutive_failures: u32,
}

impl PlasticityTracker {
    /// Create a tracker with all counters at zero.
    pub fn new() -> Self {
        Self {
            total_injections: 0,
            successes: 0,
            failures: 0,
            consecutive_failures: 0,
        }
    }

    /// Record that a recovery prompt was injected into the context.
    pub fn record_recovery_injected(&mut self) {
        self.total_injections += 1;
    }

    /// Record the observed outcome of the most recent recovery injection.
    pub fn record_outcome(&mut self, outcome: RecoveryOutcome) {
        match outcome {
            RecoveryOutcome::Success => {
                self.successes += 1;
                self.consecutive_failures = 0;
            }
            RecoveryOutcome::Failure => {
                self.failures += 1;
                self.consecutive_failures += 1;
            }
        }
    }

    /// Total number of recoveries that have been observed (successes + failures).
    pub fn total_recoveries(&self) -> u32 {
        self.successes + self.failures
    }

    /// Plasticity score: the fraction of recovery attempts that succeeded.
    ///
    /// Returns `1.0` (optimistic default) when no recoveries have been
    /// observed yet.
    pub fn plasticity_score(&self) -> f64 {
        let total = self.total_recoveries();
        if total == 0 {
            return 1.0;
        }
        self.successes as f64 / total as f64
    }

    /// Whether the developer is exhibiting "low plasticity" -- recovery
    /// prompts are not working (score < 0.3).
    pub fn is_low_plasticity(&self) -> bool {
        self.plasticity_score() < 0.3
    }

    /// Amplification level for recovery prompts based on consecutive failures.
    ///
    /// - `1` -- standard prompt (0 consecutive failures)
    /// - `2` -- emphatic prompt  (1 consecutive failure)
    /// - `3` -- escalated prompt (2+ consecutive failures)
    pub fn amplification_level(&self) -> u8 {
        match self.consecutive_failures {
            0 => 1,
            1 => 2,
            _ => 3,
        }
    }
}

impl Default for PlasticityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_impl_matches_new() {
        let a = PlasticityTracker::new();
        let b = PlasticityTracker::default();
        assert_eq!(a.total_recoveries(), b.total_recoveries());
        assert!((a.plasticity_score() - b.plasticity_score()).abs() < f64::EPSILON);
    }
}
