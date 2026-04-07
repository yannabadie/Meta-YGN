#![cfg(feature = "judge")]

use metaygn_daemon::judge::{build_judge_prompt, HaikuJudge, JudgeConfig, JudgeVerdict};

#[test]
fn judge_config_defaults() {
    let config = JudgeConfig::default();
    assert_eq!(config.max_calls, 20);
    assert_eq!(config.timeout_ms, 500);
    assert_eq!(config.cache_size, 100);
    assert_eq!(config.model, "claude-haiku-4-5-20251001");
}

#[test]
fn judge_without_api_key_returns_abstain() {
    // Temporarily ensure ANTHROPIC_API_KEY is absent (test isolation).
    // We rely on CI not setting this variable. If it IS set, the judge
    // will be "available" but the test still verifies construction works.
    let config = JudgeConfig::default();
    let judge = HaikuJudge::new(config);

    if !judge.is_available() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        let verdict = rt.block_on(judge.evaluate("rm -rf /", None));
        assert_eq!(verdict, JudgeVerdict::Abstain);
    }
    // If API key IS set we skip the assertion — we don't want to make a real
    // API call in unit tests.
}

#[test]
fn judge_budget_tracks_remaining() {
    let config = JudgeConfig {
        max_calls: 5,
        ..Default::default()
    };
    let judge = HaikuJudge::new(config);
    assert_eq!(judge.remaining_budget(), 5);
}

#[test]
fn judge_cache_returns_same_verdict() {
    let config = JudgeConfig::default();
    let judge = HaikuJudge::new(config);

    // Manually insert a cached verdict.
    judge.cache_verdict("echo hello", None, JudgeVerdict::Safe);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let verdict = rt.block_on(judge.evaluate("echo hello", None));
    assert_eq!(verdict, JudgeVerdict::Safe);
}

#[test]
fn verdict_debug_format() {
    let dbg = format!("{:?}", JudgeVerdict::Abstain);
    assert_eq!(dbg, "Abstain");
    let dbg = format!("{:?}", JudgeVerdict::Dangerous);
    assert_eq!(dbg, "Dangerous");
}

#[test]
fn prompt_template_contains_command() {
    let prompt = build_judge_prompt("rm -rf /", Some("production server"));
    assert!(prompt.contains("rm -rf /"));
    assert!(prompt.contains("production server"));
    assert!(prompt.contains("SAFE"));
    assert!(prompt.contains("RISKY"));
    assert!(prompt.contains("DANGEROUS"));
}
