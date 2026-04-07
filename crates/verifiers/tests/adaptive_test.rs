use metaygn_verifiers::adaptive::AdaptiveGuardMemory;

#[test]
fn new_memory_is_empty() {
    let mem = AdaptiveGuardMemory::new();
    assert!(mem.stats().is_empty());
    assert!(mem.active_rules().is_empty());
}

#[test]
fn record_hit_increments_count() {
    let mut mem = AdaptiveGuardMemory::new();
    mem.record_hit("no_rm_rf");
    mem.record_hit("no_rm_rf");
    mem.record_hit("no_rm_rf");
    let stats = mem.stats();
    assert_eq!(stats.len(), 1);
    // No outcomes recorded yet, so effectiveness is 0.0 and observations 0.
    assert_eq!(stats[0].2, 0);
}

#[test]
fn record_true_positive_outcome() {
    let mut mem = AdaptiveGuardMemory::new();
    mem.record_outcome("no_rm_rf", true);
    assert_eq!(mem.effectiveness("no_rm_rf"), Some(1.0));
}

#[test]
fn record_false_positive_outcome() {
    let mut mem = AdaptiveGuardMemory::new();
    mem.record_outcome("no_rm_rf", false);
    assert_eq!(mem.effectiveness("no_rm_rf"), Some(0.0));
}

#[test]
fn effectiveness_calculation() {
    let mut mem = AdaptiveGuardMemory::new();
    // 3 true positives, 1 false positive → effectiveness = 3/4 = 0.75
    mem.record_outcome("guard_a", true);
    mem.record_outcome("guard_a", true);
    mem.record_outcome("guard_a", true);
    mem.record_outcome("guard_a", false);
    assert_eq!(mem.effectiveness("guard_a"), Some(0.75));
}

#[test]
fn should_disable_low_effectiveness_enough_observations() {
    let mut mem = AdaptiveGuardMemory::new();
    // 1 TP, 9 FP → effectiveness = 0.1, observations = 10
    mem.record_outcome("noisy_rule", true);
    for _ in 0..9 {
        mem.record_outcome("noisy_rule", false);
    }
    // min_observations=5, min_effectiveness=0.5 → should disable
    assert!(mem.should_disable("noisy_rule", 5, 0.5));
}

#[test]
fn should_disable_low_effectiveness_too_few_observations() {
    let mut mem = AdaptiveGuardMemory::new();
    // 0 TP, 2 FP → effectiveness = 0.0, but only 2 observations
    mem.record_outcome("new_rule", false);
    mem.record_outcome("new_rule", false);
    // min_observations=5 → not enough data, should NOT disable
    assert!(!mem.should_disable("new_rule", 5, 0.5));
}

#[test]
fn should_disable_high_effectiveness() {
    let mut mem = AdaptiveGuardMemory::new();
    // 9 TP, 1 FP → effectiveness = 0.9
    for _ in 0..9 {
        mem.record_outcome("good_rule", true);
    }
    mem.record_outcome("good_rule", false);
    // Effectiveness 0.9 >= 0.5 → should NOT disable
    assert!(!mem.should_disable("good_rule", 5, 0.5));
}

#[test]
fn multiple_rules_tracked_independently() {
    let mut mem = AdaptiveGuardMemory::new();
    mem.record_outcome("rule_a", true);
    mem.record_outcome("rule_a", true);
    mem.record_outcome("rule_b", false);
    mem.record_outcome("rule_b", false);

    assert_eq!(mem.effectiveness("rule_a"), Some(1.0));
    assert_eq!(mem.effectiveness("rule_b"), Some(0.0));
}

#[test]
fn effectiveness_unknown_rule_returns_none() {
    let mem = AdaptiveGuardMemory::new();
    assert_eq!(mem.effectiveness("nonexistent"), None);
}

#[test]
fn active_rules_filters_low_effectiveness() {
    let mut mem = AdaptiveGuardMemory::new();
    // good_rule: 100% effective
    mem.record_outcome("good_rule", true);
    // bad_rule: 0% effective
    mem.record_outcome("bad_rule", false);
    // new_rule: no outcomes (benefit of the doubt)
    mem.record_hit("new_rule");

    let active = mem.active_rules();
    assert!(active.contains(&"good_rule"));
    assert!(!active.contains(&"bad_rule"));
    assert!(active.contains(&"new_rule"));
}
