use metaygn_shared::events::MetaEvent;
use std::collections::HashSet;

#[test]
fn event_type_matches_variant() {
    let event = MetaEvent::SessionStarted {
        stack: vec!["classify".into(), "assess".into()],
        source: "test".into(),
    };
    assert_eq!(event.event_type(), "session_started");

    let event = MetaEvent::ToolGated {
        tool: "bash".into(),
        decision: "deny".into(),
        guard: "risk_gate".into(),
        score: 90,
    };
    assert_eq!(event.event_type(), "tool_gated");

    let event = MetaEvent::SessionEnded {
        reason: "completed".into(),
    };
    assert_eq!(event.event_type(), "session_ended");
}

#[test]
fn serialization_roundtrip() {
    let event = MetaEvent::BudgetConsumed {
        tokens: 1500,
        cost_usd: 0.05,
        utilization: 0.3,
    };

    let json = event.to_json();
    let deserialized: MetaEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.event_type(), "budget_consumed");
    match deserialized {
        MetaEvent::BudgetConsumed {
            tokens,
            cost_usd,
            utilization,
        } => {
            assert_eq!(tokens, 1500);
            assert!((cost_usd - 0.05).abs() < f64::EPSILON);
            assert!((utilization - 0.3).abs() < f64::EPSILON);
        }
        _ => panic!("wrong variant after deserialization"),
    }
}

#[test]
fn all_variants_have_unique_types() {
    let events: Vec<MetaEvent> = vec![
        MetaEvent::SessionStarted { stack: vec![], source: String::new() },
        MetaEvent::PromptClassified { risk: String::new(), strategy: String::new(), topology: String::new() },
        MetaEvent::ToolGated { tool: String::new(), decision: String::new(), guard: String::new(), score: 0 },
        MetaEvent::ToolCompleted { tool: String::new(), success: true, duration_ms: 0 },
        MetaEvent::ToolFailed { tool: String::new(), error: String::new() },
        MetaEvent::RecoveryInjected { level: 0, reason: String::new() },
        MetaEvent::RecoveryOutcome { success: true, plasticity_score: 0.0 },
        MetaEvent::CompletionVerified { verified: true, issues: vec![] },
        MetaEvent::TestIntegrityWarning { file: String::new(), issues: vec![] },
        MetaEvent::BudgetConsumed { tokens: 0, cost_usd: 0.0, utilization: 0.0 },
        MetaEvent::SessionEnded { reason: String::new() },
    ];

    let mut types: HashSet<&str> = HashSet::new();
    for event in &events {
        let t = event.event_type();
        assert!(
            types.insert(t),
            "duplicate event_type: {}",
            t
        );
    }

    assert_eq!(types.len(), 11, "expected 11 unique event types");
}
