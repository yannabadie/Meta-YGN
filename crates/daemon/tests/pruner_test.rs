use metaygn_daemon::proxy::pruner::{ContextPruner, Message};

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

    let user_messages: Vec<&Message> = pruned.iter().filter(|m| m.role == "user").collect();
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
    let has_injection = pruned.iter().any(|m| {
        m.content.contains("ALETHEIA") && m.content.contains("fundamentally different approach")
    });
    assert!(
        has_injection,
        "Recovery injection not found in pruned messages"
    );
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
    // Only the last two are consecutive errors — the success message breaks the streak.
    assert_eq!(analysis.consecutive_errors, 2);
    assert!(!analysis.should_prune);
}

#[test]
fn amplified_recovery_level_2_is_emphatic() {
    let pruner = ContextPruner::with_defaults();
    let msg = pruner.amplified_recovery("reasoning loop", 2);
    assert!(msg.contains("CRITICAL"));
    assert!(msg.contains("different approach"));
}

#[test]
fn amplified_recovery_level_3_escalates() {
    let pruner = ContextPruner::with_defaults();
    let msg = pruner.amplified_recovery("reasoning loop", 3);
    assert!(msg.contains("ESCALATE"));
    assert!(msg.contains("/metacog-escalate"));
}
