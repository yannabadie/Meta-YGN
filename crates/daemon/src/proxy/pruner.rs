use serde::{Deserialize, Serialize};

/// A single message in the Anthropic messages format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Result of analyzing a message history for reasoning lock-in.
#[derive(Debug)]
pub struct PruneAnalysis {
    pub consecutive_errors: usize,
    pub should_prune: bool,
    pub error_indices: Vec<usize>,
    pub suggested_injection: Option<String>,
}

/// Configuration for the pruner.
pub struct PrunerConfig {
    pub error_threshold: usize,
    pub error_patterns: Vec<String>,
}

impl Default for PrunerConfig {
    fn default() -> Self {
        Self {
            error_threshold: 3,
            error_patterns: vec![
                "error".into(),
                "Error".into(),
                "ERROR".into(),
                "failed".into(),
                "Failed".into(),
                "FAILED".into(),
                "traceback".into(),
                "Traceback".into(),
                "panic".into(),
                "exception".into(),
                "Exception".into(),
                "cannot".into(),
                "Cannot".into(),
                "not found".into(),
                "Not found".into(),
                "permission denied".into(),
                "compilation failed".into(),
                "test failed".into(),
                "tests failed".into(),
            ],
        }
    }
}

/// Context pruner that detects reasoning lock-in and amputates failed context.
///
/// Reasoning lock-in occurs when an assistant produces 3+ consecutive error
/// responses, indicating it is stuck in a failing loop. The pruner removes
/// those error messages and injects a recovery prompt to break the cycle.
pub struct ContextPruner {
    config: PrunerConfig,
}

impl ContextPruner {
    pub fn new(config: PrunerConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(PrunerConfig::default())
    }

    /// Check whether a message contains any error pattern.
    fn is_error_message(&self, message: &Message) -> bool {
        let content = &message.content;
        self.config
            .error_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
    }

    /// Build a short summary of the errors that were pruned.
    fn summarize_errors(&self, messages: &[Message], indices: &[usize]) -> String {
        // Collect unique error patterns found across the pruned messages.
        let mut found_patterns: Vec<&str> = Vec::new();
        for &idx in indices {
            if let Some(msg) = messages.get(idx) {
                for pattern in &self.config.error_patterns {
                    if msg.content.contains(pattern.as_str()) && !found_patterns.contains(&pattern.as_str()) {
                        found_patterns.push(pattern.as_str());
                    }
                }
            }
        }
        if found_patterns.is_empty() {
            "repeated errors".to_string()
        } else {
            found_patterns.join(", ")
        }
    }

    /// Analyze a message history for reasoning lock-in.
    ///
    /// Scans from the end of the message list, counting consecutive assistant
    /// messages whose content matches one or more error patterns. User messages
    /// are skipped (they don't break the streak), but a successful assistant
    /// message does break it. If the count meets or exceeds `error_threshold`,
    /// the analysis recommends pruning.
    pub fn analyze(&self, messages: &[Message]) -> PruneAnalysis {
        let mut consecutive_errors: usize = 0;
        let mut error_indices: Vec<usize> = Vec::new();

        // Scan backwards from the end, looking for consecutive assistant errors.
        // User messages are transparent — they don't break the streak.
        for i in (0..messages.len()).rev() {
            let msg = &messages[i];

            if msg.role != "assistant" {
                // Skip non-assistant messages (user, system) — they don't
                // break the consecutive assistant error streak.
                continue;
            }

            if self.is_error_message(msg) {
                consecutive_errors += 1;
                error_indices.push(i);
            } else {
                // A successful assistant message breaks the streak.
                break;
            }
        }

        // Reverse so indices are in ascending order.
        error_indices.reverse();

        let should_prune = consecutive_errors >= self.config.error_threshold;

        let suggested_injection = if should_prune {
            let summary = self.summarize_errors(messages, &error_indices);
            Some(format!(
                "[ALETHEIA: Context pruned. {} failed reasoning attempts removed. \
                 Previous approaches failed due to: {}. \
                 Start with a fundamentally different approach.]",
                consecutive_errors, summary
            ))
        } else {
            None
        };

        PruneAnalysis {
            consecutive_errors,
            should_prune,
            error_indices,
            suggested_injection,
        }
    }

    /// Prune error messages from the history and inject a recovery prompt.
    ///
    /// Rules:
    /// - If analysis says no pruning needed, return messages unchanged.
    /// - Never prune user messages.
    /// - Keep the first and last messages intact.
    /// - Only prune consecutive assistant error messages identified by analyze().
    /// - Inject a recovery system message where the errors were removed.
    pub fn prune(&self, messages: &[Message]) -> Vec<Message> {
        let analysis = self.analyze(messages);

        if !analysis.should_prune {
            return messages.to_vec();
        }

        // Build the set of indices to remove.
        // Protect: first message, last message, and any user messages.
        let last_idx = messages.len().saturating_sub(1);
        let removable: Vec<usize> = analysis
            .error_indices
            .iter()
            .copied()
            .filter(|&i| {
                i != 0
                    && i != last_idx
                    && messages[i].role != "user"
            })
            .collect();

        let mut result: Vec<Message> = Vec::with_capacity(messages.len());
        let mut injection_placed = false;

        for (i, msg) in messages.iter().enumerate() {
            if removable.contains(&i) {
                // Insert the recovery injection once, at the position of the
                // first removed message.
                if !injection_placed {
                    if let Some(ref injection) = analysis.suggested_injection {
                        result.push(Message {
                            role: "assistant".to_string(),
                            content: injection.clone(),
                        });
                        injection_placed = true;
                    }
                }
                // Skip the error message.
                continue;
            }
            result.push(msg.clone());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a message.
    fn msg(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn no_errors_no_prune() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Hello"),
            msg("assistant", "Hi there! How can I help?"),
            msg("user", "Write a function"),
            msg("assistant", "Here is the function: fn foo() {}"),
        ];
        let analysis = pruner.analyze(&messages);
        assert!(!analysis.should_prune);
        assert_eq!(analysis.consecutive_errors, 0);
        assert!(analysis.error_indices.is_empty());
        assert!(analysis.suggested_injection.is_none());
    }

    #[test]
    fn two_errors_below_threshold() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Fix the bug"),
            msg("assistant", "Error: compilation failed at line 5"),
            msg("assistant", "Error: compilation failed at line 10"),
        ];
        let analysis = pruner.analyze(&messages);
        assert!(!analysis.should_prune);
        assert_eq!(analysis.consecutive_errors, 2);
    }

    #[test]
    fn three_errors_triggers_prune() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Fix the bug"),
            msg("assistant", "Error: compilation failed"),
            msg("assistant", "Error: test failed again"),
            msg("assistant", "Error: cannot resolve dependency"),
        ];
        let analysis = pruner.analyze(&messages);
        assert!(analysis.should_prune);
        assert_eq!(analysis.consecutive_errors, 3);
        assert_eq!(analysis.error_indices, vec![1, 2, 3]);
        assert!(analysis.suggested_injection.is_some());
        let injection = analysis.suggested_injection.unwrap();
        assert!(injection.contains("ALETHEIA"));
        assert!(injection.contains("3 failed reasoning attempts removed"));
    }

    #[test]
    fn prune_removes_error_messages() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Fix the bug"),
            msg("assistant", "Error: compilation failed"),
            msg("assistant", "Error: test failed again"),
            msg("assistant", "Error: cannot resolve dependency"),
            msg("user", "Try a different approach"),
        ];
        let pruned = pruner.prune(&messages);

        // The three error assistant messages in the middle should be replaced
        // by a single injection message.
        for m in &pruned {
            if m.role == "assistant" {
                // Only the injected recovery message should remain.
                assert!(
                    m.content.contains("ALETHEIA"),
                    "Expected ALETHEIA injection, got: {}",
                    m.content
                );
            }
        }
        // Both user messages must survive.
        let user_count = pruned.iter().filter(|m| m.role == "user").count();
        assert_eq!(user_count, 2);
    }

    #[test]
    fn prune_preserves_user_messages() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Step one"),
            msg("assistant", "Error: failed step one"),
            msg("assistant", "Error: failed retry"),
            msg("assistant", "Error: failed again"),
            msg("user", "Step two"),
        ];
        let pruned = pruner.prune(&messages);

        let user_messages: Vec<&Message> =
            pruned.iter().filter(|m| m.role == "user").collect();
        assert_eq!(user_messages.len(), 2);
        assert_eq!(user_messages[0].content, "Step one");
        assert_eq!(user_messages[1].content, "Step two");
    }

    #[test]
    fn prune_injects_recovery_prompt() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Fix the bug"),
            msg("assistant", "Error: compilation failed"),
            msg("assistant", "Error: test failed"),
            msg("assistant", "Error: cannot build"),
            msg("user", "Any ideas?"),
        ];
        let pruned = pruner.prune(&messages);
        let has_injection = pruned
            .iter()
            .any(|m| m.content.contains("ALETHEIA") && m.content.contains("fundamentally different approach"));
        assert!(has_injection, "Recovery injection not found in pruned messages");
    }

    #[test]
    fn non_consecutive_errors_dont_trigger() {
        let pruner = ContextPruner::with_defaults();
        let messages = vec![
            msg("user", "Fix the bug"),
            msg("assistant", "Error: compilation failed"),
            msg("assistant", "OK, I fixed that issue."),
            msg("assistant", "Error: test failed"),
            msg("assistant", "Error: cannot resolve"),
        ];
        let analysis = pruner.analyze(&messages);
        // Only the last two are consecutive errors — the "OK" message breaks the streak.
        assert_eq!(analysis.consecutive_errors, 2);
        assert!(!analysis.should_prune);
    }
}
