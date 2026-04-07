//! SequenceMonitor — DTMC-inspired multi-action pattern detector.
//!
//! Tracks sequences of agent actions within a sliding window and fires alerts
//! when dangerous multi-step patterns are detected. Inspired by Pro2Guard
//! (arxiv 2508.00500). Instead of a full DTMC with PCTL model checking, v2.2
//! uses rule-based pattern matching on a sliding window of symbolic action
//! states.

use std::collections::{HashMap, HashSet};

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// The kind of action the agent performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    Read,
    Write,
    Delete,
    Execute,
    NetworkRead,
    GitPush,
    Error,
    Privilege,
    Unknown,
}

/// What the action targeted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetType {
    File,
    TestFile,
    SensitivePath,
    Url,
    Remote,
    Unknown,
}

/// A single symbolic action state recorded by the monitor.
#[derive(Debug, Clone)]
pub struct ActionState {
    pub action_type: ActionType,
    pub target_type: TargetType,
    pub tool_name: String,
    pub detail: String,
}

/// An alert fired when a dangerous multi-step pattern is detected.
#[derive(Debug, Clone)]
pub struct SequenceAlert {
    /// Name of the rule that fired (e.g. `"network_then_sensitive_write"`).
    pub rule: String,
    /// Human-readable description of the detected pattern.
    pub description: String,
    /// Indices (within the current window) of the actions that matched.
    pub action_indices: Vec<usize>,
}

// ──────────────────────────────────────────────────────────────
// SequenceMonitor
// ──────────────────────────────────────────────────────────────

/// Sliding-window pattern detector for agent action sequences.
pub struct SequenceMonitor {
    /// Maximum number of actions retained in the window.
    max_window: usize,
    /// Current sliding window of actions.
    window: Vec<ActionState>,
    /// Total number of actions ever recorded (not just in window).
    total_count: usize,
    /// Transition counts across ALL recorded actions (never evicted).
    transitions: HashMap<(ActionType, ActionType), usize>,
    /// The last action type seen (for computing transitions).
    last_action_type: Option<ActionType>,
    /// Alerts that have fired.
    alerts: Vec<SequenceAlert>,
    /// Set of rule names that have already fired (each fires at most once).
    fired_rules: HashSet<String>,
}

impl SequenceMonitor {
    /// Create a new monitor with the default window size of 20.
    pub fn new() -> Self {
        Self::with_window(20)
    }

    /// Create a new monitor with a custom window size.
    pub fn with_window(max: usize) -> Self {
        Self {
            max_window: max,
            window: Vec::with_capacity(max),
            total_count: 0,
            transitions: HashMap::new(),
            last_action_type: None,
            alerts: Vec::new(),
            fired_rules: HashSet::new(),
        }
    }

    /// Record a new action. Evicts the oldest action if the window is full,
    /// updates transition counts, then checks all safety rules.
    pub fn record(&mut self, action: ActionState) {
        // Track transitions across ALL actions (not just window).
        if let Some(prev) = self.last_action_type {
            *self.transitions.entry((prev, action.action_type)).or_insert(0) += 1;
        }
        self.last_action_type = Some(action.action_type);
        self.total_count += 1;

        // Evict oldest if window is full.
        if self.window.len() >= self.max_window {
            self.window.remove(0);
        }
        self.window.push(action);

        self.check_rules();
    }

    /// Total number of actions ever recorded (including evicted ones).
    pub fn action_count(&self) -> usize {
        self.total_count
    }

    /// All alerts fired so far.
    pub fn alerts(&self) -> &[SequenceAlert] {
        &self.alerts
    }

    /// How many times the transition `from -> to` has been observed across all
    /// recorded actions (not limited to the current window).
    pub fn transition_count(&self, from: ActionType, to: ActionType) -> usize {
        self.transitions.get(&(from, to)).copied().unwrap_or(0)
    }

    // ──────────────────────────────────────────────────────────
    // Rule checking
    // ──────────────────────────────────────────────────────────

    fn check_rules(&mut self) {
        self.check_network_then_sensitive_write();
        self.check_delete_then_force_push();
        self.check_errors_then_test_modify();
    }

    /// Rule 1: `network_then_sensitive_write`
    ///
    /// NetworkRead anywhere in window, followed later by Write to SensitivePath.
    fn check_network_then_sensitive_write(&mut self) {
        let rule = "network_then_sensitive_write";
        if self.fired_rules.contains(rule) {
            return;
        }

        // Find earliest NetworkRead and latest Write(SensitivePath) where
        // the network read comes before the sensitive write.
        let network_idx = self.window.iter().position(|a| a.action_type == ActionType::NetworkRead);
        let write_idx = self.window.iter().rposition(|a| {
            a.action_type == ActionType::Write && a.target_type == TargetType::SensitivePath
        });

        if let (Some(ni), Some(wi)) = (network_idx, write_idx)
            && ni < wi
        {
            self.fired_rules.insert(rule.to_string());
            self.alerts.push(SequenceAlert {
                rule: rule.to_string(),
                description: "NetworkRead followed by Write to SensitivePath detected"
                    .to_string(),
                action_indices: vec![ni, wi],
            });
        }
    }

    /// Rule 2: `delete_then_force_push`
    ///
    /// Delete anywhere in window, followed later by GitPush.
    fn check_delete_then_force_push(&mut self) {
        let rule = "delete_then_force_push";
        if self.fired_rules.contains(rule) {
            return;
        }

        let delete_idx = self.window.iter().position(|a| a.action_type == ActionType::Delete);
        let push_idx = self
            .window
            .iter()
            .rposition(|a| a.action_type == ActionType::GitPush);

        if let (Some(di), Some(pi)) = (delete_idx, push_idx)
            && di < pi
        {
            self.fired_rules.insert(rule.to_string());
            self.alerts.push(SequenceAlert {
                rule: rule.to_string(),
                description: "Delete followed by GitPush detected".to_string(),
                action_indices: vec![di, pi],
            });
        }
    }

    /// Rule 3: `errors_then_test_modify`
    ///
    /// 3+ consecutive Error actions immediately before a Write to TestFile.
    fn check_errors_then_test_modify(&mut self) {
        let rule = "errors_then_test_modify";
        if self.fired_rules.contains(rule) {
            return;
        }

        let len = self.window.len();
        if len < 4 {
            return; // Need at least 3 errors + 1 write
        }

        // Check if the last action is a Write to TestFile.
        let last = &self.window[len - 1];
        if last.action_type != ActionType::Write || last.target_type != TargetType::TestFile {
            return;
        }

        // Count consecutive errors immediately before the write.
        let mut error_count = 0;
        for i in (0..len - 1).rev() {
            if self.window[i].action_type == ActionType::Error {
                error_count += 1;
            } else {
                break;
            }
        }

        if error_count >= 3 {
            let start = len - 1 - error_count;
            let indices: Vec<usize> = (start..len).collect();
            self.fired_rules.insert(rule.to_string());
            self.alerts.push(SequenceAlert {
                rule: rule.to_string(),
                description: format!(
                    "{} consecutive errors followed by Write to TestFile",
                    error_count
                ),
                action_indices: indices,
            });
        }
    }
}

impl Default for SequenceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(action_type: ActionType, target_type: TargetType) -> ActionState {
        ActionState {
            action_type,
            target_type,
            tool_name: String::new(),
            detail: String::new(),
        }
    }

    #[test]
    fn empty_monitor_has_no_alerts() {
        let mon = SequenceMonitor::new();
        assert!(mon.alerts().is_empty());
        assert_eq!(mon.action_count(), 0);
    }

    #[test]
    fn record_action_increments_count() {
        let mut mon = SequenceMonitor::new();
        mon.record(action(ActionType::Read, TargetType::File));
        assert_eq!(mon.action_count(), 1);
    }

    #[test]
    fn network_read_then_sensitive_write_triggers_alert() {
        let mut mon = SequenceMonitor::new();
        mon.record(action(ActionType::NetworkRead, TargetType::Url));
        mon.record(action(ActionType::Write, TargetType::SensitivePath));
        assert_eq!(mon.alerts().len(), 1);
        assert!(mon.alerts()[0].rule.contains("network_then_sensitive_write"));
    }

    #[test]
    fn delete_then_force_push_triggers_alert() {
        let mut mon = SequenceMonitor::new();
        mon.record(action(ActionType::Delete, TargetType::File));
        mon.record(action(ActionType::GitPush, TargetType::Remote));
        assert_eq!(mon.alerts().len(), 1);
        assert!(mon.alerts()[0].rule.contains("delete_then_force_push"));
    }

    #[test]
    fn repeated_errors_then_test_modify_triggers_alert() {
        let mut mon = SequenceMonitor::new();
        for _ in 0..3 {
            mon.record(action(ActionType::Error, TargetType::Unknown));
        }
        mon.record(action(ActionType::Write, TargetType::TestFile));
        assert_eq!(mon.alerts().len(), 1);
        assert!(mon.alerts()[0].rule.contains("errors_then_test_modify"));
    }

    #[test]
    fn safe_read_sequence_no_alerts() {
        let mut mon = SequenceMonitor::new();
        for _ in 0..10 {
            mon.record(action(ActionType::Read, TargetType::File));
        }
        assert!(mon.alerts().is_empty());
    }

    #[test]
    fn write_then_test_is_normal() {
        let mut mon = SequenceMonitor::new();
        mon.record(action(ActionType::Write, TargetType::File));
        mon.record(action(ActionType::Execute, TargetType::File));
        assert!(mon.alerts().is_empty());
    }

    #[test]
    fn alert_fires_once_not_repeatedly() {
        let mut mon = SequenceMonitor::new();
        mon.record(action(ActionType::NetworkRead, TargetType::Url));
        mon.record(action(ActionType::Write, TargetType::SensitivePath));
        assert_eq!(mon.alerts().len(), 1);
        // More benign actions should not cause a second alert.
        for _ in 0..5 {
            mon.record(action(ActionType::Read, TargetType::File));
        }
        mon.record(action(ActionType::NetworkRead, TargetType::Url));
        mon.record(action(ActionType::Write, TargetType::SensitivePath));
        assert_eq!(mon.alerts().len(), 1);
    }

    #[test]
    fn old_actions_beyond_window_dont_trigger() {
        let mut mon = SequenceMonitor::with_window(5);
        mon.record(action(ActionType::NetworkRead, TargetType::Url));
        // Push 10 benign reads to evict the NetworkRead.
        for _ in 0..10 {
            mon.record(action(ActionType::Read, TargetType::File));
        }
        // Now write to SensitivePath — should NOT trigger since
        // NetworkRead has been evicted from the window.
        mon.record(action(ActionType::Write, TargetType::SensitivePath));
        assert!(mon.alerts().is_empty());
    }
}
