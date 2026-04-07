#![cfg(feature = "semantic")]

use std::sync::Arc;

use metaygn_daemon::semantic_router::{LabeledExample, RiskTier, SemanticRouter};
use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider};
use metaygn_shared::state::RoutingHint;

fn make_router() -> SemanticRouter {
    let embed: Arc<dyn EmbeddingProvider> = Arc::new(HashEmbedProvider::new(128));
    SemanticRouter::new(embed)
}

#[test]
fn router_has_initial_examples() {
    let router = make_router();
    // 16 safe + 11 dangerous + 10 ambiguous = 37
    assert!(
        router.example_count() >= 36,
        "expected >= 36 seeded examples, got {}",
        router.example_count()
    );
}

#[test]
fn add_example_increases_count() {
    let router = make_router();
    let before = router.example_count();
    router.add_example(LabeledExample {
        command: "new-command --flag".to_string(),
        context: Some("test context".to_string()),
        tier: RiskTier::Ambiguous,
    });
    assert_eq!(router.example_count(), before + 1);
}

#[test]
fn classify_returns_valid_risk_tier() {
    let router = make_router();
    for cmd in &["ls", "rm -rf /", "git push --force", "cargo test"] {
        let tier = router.classify(cmd, None);
        assert!(
            tier == RiskTier::Safe || tier == RiskTier::Ambiguous || tier == RiskTier::Dangerous,
            "unexpected tier for `{cmd}`: {tier:?}"
        );
    }
}

#[test]
fn routing_hint_returns_valid_hint() {
    let router = make_router();
    for cmd in &["echo hello", "sudo rm -rf /", "terraform apply"] {
        let hint = router.routing_hint(cmd, None);
        assert!(
            hint == RoutingHint::Deterministic
                || hint == RoutingHint::LlmJudge
                || matches!(hint, RoutingHint::SemanticMatch { .. })
                || hint == RoutingHint::SequenceCheck,
            "unexpected hint for `{cmd}`: {hint:?}"
        );
    }
}

#[test]
fn hash_embeddings_do_not_crash_on_any_input() {
    let router = make_router();
    let long = "x ".repeat(500);
    let inputs = vec![
        "",
        " ",
        "a",
        "rm",
        "rm -rf /",
        "ls -la",
        "git push --force origin main",
        ":(){ :|:& };:",
        long.as_str(),
    ];
    for input in inputs {
        // Must not panic.
        let _ = router.classify(input, None);
        let _ = router.routing_hint(input, None);
        let _ = router.classify(input, Some("some context"));
    }
}

#[test]
fn classify_with_context_does_not_panic() {
    let router = make_router();
    let tier = router.classify("rm -rf /tmp/build", Some("CI cleanup step"));
    assert!(
        tier == RiskTier::Safe || tier == RiskTier::Ambiguous || tier == RiskTier::Dangerous
    );
}
