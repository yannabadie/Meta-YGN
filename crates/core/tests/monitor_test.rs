use chrono::Utc;
use metaygn_core::monitor::{
    cosine_similarity, AnomalyReport, MascMonitor, MonitorConfig, ReasoningStep,
};
use std::collections::HashMap;

/// Helper to quickly build a [`ReasoningStep`].
fn step(n: usize, content: &str) -> ReasoningStep {
    ReasoningStep {
        content: content.to_string(),
        step_number: n,
        timestamp: Utc::now(),
    }
}

// ──────────────────────────────────────────────────────────────
// Integration tests
// ──────────────────────────────────────────────────────────────

#[test]
fn empty_history_no_anomaly() {
    let mut monitor = MascMonitor::with_defaults();
    let report = monitor.observe(step(1, "Begin analyzing the codebase structure"));

    assert!(
        !report.is_anomalous,
        "The very first step should never be flagged as anomalous"
    );
    assert!(
        (report.similarity_score - 1.0).abs() < f64::EPSILON,
        "First step similarity should be 1.0"
    );
}

#[test]
fn similar_steps_no_anomaly() {
    let mut monitor = MascMonitor::with_defaults();

    let topics = [
        "Analyzing the authentication module for security vulnerabilities",
        "Reviewing the authentication service for potential security issues",
        "Checking the auth module code for security weaknesses and flaws",
        "Examining authentication logic to find security vulnerabilities",
        "Auditing the authentication system for security problems",
    ];

    let mut last_report: Option<AnomalyReport> = None;
    for (i, topic) in topics.iter().enumerate() {
        last_report = Some(monitor.observe(step(i + 1, topic)));
    }

    let report = last_report.unwrap();
    assert!(
        !report.is_anomalous,
        "Consistent reasoning about the same topic should not be anomalous, \
         similarity={:.3}, threshold={:.3}",
        report.similarity_score, report.threshold,
    );
}

#[test]
fn completely_different_step_is_anomaly() {
    let mut monitor = MascMonitor::new(MonitorConfig {
        window_size: 10,
        anomaly_threshold: 0.15,
        stagnation_threshold: 0.95,
    });

    // Build a consistent history about Rust programming.
    for i in 1..=5 {
        monitor.observe(step(
            i,
            "Implementing the Rust borrow checker lifetime analysis module",
        ));
    }

    // Now introduce a completely unrelated step about cooking.
    let report = monitor.observe(step(
        6,
        "Preheat the oven to 350 degrees and mix flour with sugar for the cake batter",
    ));

    assert!(
        report.is_anomalous,
        "A sudden topic change should be flagged as anomalous, \
         similarity={:.3}, threshold={:.3}",
        report.similarity_score, report.threshold,
    );
    assert!(
        report.reason.is_some(),
        "Anomaly report should contain a reason"
    );
    assert!(
        report.similarity_score < report.threshold,
        "Similarity {:.3} should be below anomaly threshold {:.3}",
        report.similarity_score,
        report.threshold,
    );
}

#[test]
fn repeated_step_is_stagnation() {
    let mut monitor = MascMonitor::new(MonitorConfig {
        window_size: 10,
        anomaly_threshold: 0.15,
        stagnation_threshold: 0.95,
    });

    let repeated = "Checking the function for errors and validating input parameters";

    // Feed the exact same step multiple times.
    let mut last_report: Option<AnomalyReport> = None;
    for i in 1..=6 {
        last_report = Some(monitor.observe(step(i, repeated)));
    }

    let report = last_report.unwrap();
    assert!(
        report.is_anomalous,
        "Exact repetition should be flagged as stagnation, \
         similarity={:.3}, threshold={:.3}",
        report.similarity_score, report.threshold,
    );
    assert!(
        report.similarity_score > 0.95,
        "Repeated identical steps should have very high similarity ({:.3})",
        report.similarity_score,
    );
    assert!(
        report
            .reason
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains("stagnant"),
        "Reason should mention stagnation, got: {:?}",
        report.reason,
    );
}

#[test]
fn cosine_similarity_identical() {
    let mut a: HashMap<String, f64> = HashMap::new();
    a.insert("rust".into(), 0.5);
    a.insert("code".into(), 0.3);
    a.insert("module".into(), 0.8);

    let sim = cosine_similarity(&a, &a);
    assert!(
        (sim - 1.0).abs() < 1e-9,
        "Identical vectors should have cosine similarity 1.0, got {:.6}",
        sim,
    );
}

#[test]
fn cosine_similarity_orthogonal() {
    let mut a: HashMap<String, f64> = HashMap::new();
    a.insert("rust".into(), 1.0);
    a.insert("code".into(), 1.0);

    let mut b: HashMap<String, f64> = HashMap::new();
    b.insert("cake".into(), 1.0);
    b.insert("flour".into(), 1.0);

    let sim = cosine_similarity(&a, &b);
    assert!(
        sim.abs() < 1e-9,
        "Completely disjoint vectors should have cosine similarity ~0.0, got {:.6}",
        sim,
    );
}
