//! Adaptive Guard Memory — AGrail-inspired rule effectiveness tracking.
//!
//! Tracks guard rule effectiveness from session feedback without requiring
//! an LLM. Each guard rule accumulates hit counts, true-positive, and
//! false-positive tallies. The system can then recommend disabling rules
//! that consistently produce false positives, reducing noise over time.

use std::collections::HashMap;

/// Effectiveness record for a single guard rule.
#[derive(Debug, Clone)]
pub struct GuardEffectiveness {
    pub rule_name: String,
    pub hit_count: u32,
    pub true_positive: u32,
    pub false_positive: u32,
    pub last_updated: String,
}

impl GuardEffectiveness {
    fn new(rule_name: &str) -> Self {
        Self {
            rule_name: rule_name.to_string(),
            hit_count: 0,
            true_positive: 0,
            false_positive: 0,
            last_updated: String::new(),
        }
    }

    /// Precision: TP / (TP + FP). Returns `None` if there are no outcomes.
    fn effectiveness(&self) -> Option<f64> {
        let total = self.true_positive + self.false_positive;
        if total == 0 {
            return None;
        }
        Some(self.true_positive as f64 / total as f64)
    }
}

/// Tracks guard-rule effectiveness across sessions.
#[derive(Debug, Clone)]
pub struct AdaptiveGuardMemory {
    entries: HashMap<String, GuardEffectiveness>,
}

impl AdaptiveGuardMemory {
    /// Create an empty memory.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Record that a guard rule fired.
    pub fn record_hit(&mut self, rule_name: &str) {
        let entry = self
            .entries
            .entry(rule_name.to_string())
            .or_insert_with(|| GuardEffectiveness::new(rule_name));
        entry.hit_count += 1;
        entry.last_updated = now_iso();
    }

    /// Record the outcome of a previously-fired rule.
    ///
    /// - `was_harmful = true` means the flagged action was actually harmful (true positive).
    /// - `was_harmful = false` means the flagged action was actually safe (false positive).
    pub fn record_outcome(&mut self, rule_name: &str, was_harmful: bool) {
        let entry = self
            .entries
            .entry(rule_name.to_string())
            .or_insert_with(|| GuardEffectiveness::new(rule_name));
        if was_harmful {
            entry.true_positive += 1;
        } else {
            entry.false_positive += 1;
        }
        entry.last_updated = now_iso();
    }

    /// Get the effectiveness (precision) of a rule: TP / (TP + FP).
    /// Returns `None` if no outcome data exists for the rule.
    pub fn effectiveness(&self, rule_name: &str) -> Option<f64> {
        self.entries.get(rule_name).and_then(|e| e.effectiveness())
    }

    /// Whether a rule should be disabled based on accumulated evidence.
    ///
    /// Returns `true` when the rule has at least `min_observations` outcomes
    /// (TP + FP) AND its effectiveness is below `min_effectiveness`.
    pub fn should_disable(
        &self,
        rule_name: &str,
        min_observations: u32,
        min_effectiveness: f64,
    ) -> bool {
        match self.entries.get(rule_name) {
            Some(entry) => {
                let total = entry.true_positive + entry.false_positive;
                if total < min_observations {
                    return false;
                }
                match entry.effectiveness() {
                    Some(eff) => eff < min_effectiveness,
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Return rule names whose effectiveness is above a default threshold (0.5)
    /// or that have no outcome data yet (benefit of the doubt).
    pub fn active_rules(&self) -> Vec<&str> {
        self.entries
            .values()
            .filter(|e| match e.effectiveness() {
                Some(eff) => eff >= 0.5,
                None => true,
            })
            .map(|e| e.rule_name.as_str())
            .collect()
    }

    /// Return statistics for all tracked rules: (rule_name, effectiveness, total_observations).
    /// Rules with no outcomes report effectiveness as 0.0 and observations as 0.
    pub fn stats(&self) -> Vec<(&str, f64, u32)> {
        self.entries
            .values()
            .map(|e| {
                let total = e.true_positive + e.false_positive;
                let eff = e.effectiveness().unwrap_or(0.0);
                (e.rule_name.as_str(), eff, total)
            })
            .collect()
    }
}

impl Default for AdaptiveGuardMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the current time as an ISO 8601 string.
fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
