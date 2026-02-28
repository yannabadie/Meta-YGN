use metaygn_core::heuristics::evolver::HeuristicEvolver;
use metaygn_core::heuristics::fitness::{FitnessScore, SessionOutcome};

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

fn make_outcome(session_id: &str, success: bool, tokens: u64, duration_ms: u64) -> SessionOutcome {
    SessionOutcome {
        session_id: session_id.into(),
        task_type: "bugfix".into(),
        risk_level: "medium".into(),
        strategy_used: "vertical".into(),
        success,
        tokens_consumed: tokens,
        duration_ms,
        errors_encountered: if success { 0 } else { 2 },
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[test]
fn fitness_composite_calculation() {
    // Verify the weighted average: 0.5 * success + 0.3 * tokens + 0.2 * latency.
    let f = FitnessScore::compute(0.8, 0.6, 0.4);
    // Expected: 0.8*0.5 + 0.6*0.3 + 0.4*0.2 = 0.40 + 0.18 + 0.08 = 0.66
    assert!(
        (f.composite - 0.66).abs() < 1e-9,
        "composite should be 0.66, got {}",
        f.composite
    );
    assert!((f.verification_success_rate - 0.8).abs() < 1e-9);
    assert!((f.token_efficiency - 0.6).abs() < 1e-9);
    assert!((f.latency_score - 0.4).abs() < 1e-9);
}

#[test]
fn record_outcome_stores() {
    let mut evolver = HeuristicEvolver::new(20);
    evolver.record_outcome(make_outcome("s1", true, 5000, 10_000));
    evolver.record_outcome(make_outcome("s2", false, 8000, 20_000));
    evolver.record_outcome(make_outcome("s3", true, 3000, 5_000));
    assert_eq!(evolver.outcomes().len(), 3);
}

#[test]
fn mutate_produces_child() {
    let mut evolver = HeuristicEvolver::new(20);
    let parent_id = evolver.best().unwrap().id.clone();
    let parent_gen = evolver.best().unwrap().generation;

    let child = evolver.mutate_best().expect("should produce a child");

    assert_ne!(child.id, parent_id, "child should have a different id");
    assert_eq!(
        child.generation,
        parent_gen + 1,
        "child generation should be bumped"
    );
    assert_eq!(
        child.parent_id.as_deref(),
        Some(parent_id.as_str()),
        "child should reference parent"
    );
    // Population should now have seed + child = 2.
    assert_eq!(evolver.population_size(), 2);
}

#[test]
fn evaluate_updates_fitness() {
    let mut evolver = HeuristicEvolver::new(20);

    // Record several outcomes.
    for i in 0..5 {
        evolver.record_outcome(make_outcome(
            &format!("s{i}"),
            i % 2 == 0, // 3 successes out of 5
            10_000,
            30_000,
        ));
    }

    // Before evaluation, fitness should be zero (the seed default).
    assert_eq!(evolver.best().unwrap().fitness.composite, 0.0);

    evolver.evaluate_all();

    // After evaluation, fitness should be > 0 (we had successes, tokens < max,
    // duration < max).
    let fitness = &evolver.best().unwrap().fitness;
    assert!(
        fitness.composite > 0.0,
        "composite fitness should be > 0 after evaluation, got {}",
        fitness.composite
    );
    assert!(fitness.verification_success_rate > 0.0);
    assert!(fitness.token_efficiency > 0.0);
    assert!(fitness.latency_score > 0.0);
}

#[test]
fn evolve_generation_selects_best() {
    let mut evolver = HeuristicEvolver::new(20);

    // Record a mix of outcomes.
    for i in 0..10 {
        evolver.record_outcome(make_outcome(
            &format!("s{i}"),
            i < 7, // 7 successes
            20_000,
            60_000,
        ));
    }

    let best = evolver.evolve_generation().expect("should return best");

    // The best should have the highest composite fitness in the population.
    let best_composite = best.fitness.composite;
    for v in evolver.population() {
        assert!(
            v.fitness.composite <= best_composite + 1e-9,
            "best composite ({}) should be >= all others ({})",
            best_composite,
            v.fitness.composite
        );
    }

    // Generation should have advanced.
    assert!(evolver.generation() >= 1, "generation should be >= 1");
}

#[test]
fn population_stays_within_bounds() {
    let max_pop = 5;
    let mut evolver = HeuristicEvolver::new(max_pop);

    // Record some outcomes so evaluation is meaningful.
    for i in 0..10 {
        evolver.record_outcome(make_outcome(
            &format!("s{i}"),
            i % 3 != 0,
            15_000,
            45_000,
        ));
    }

    // Run many evolution cycles.
    for _ in 0..30 {
        evolver.evolve_generation();
    }

    // Population should never exceed max_population + 1 (the newly mutated
    // child is added after truncation within evolve_generation, so the size
    // is at most max_population + 1 at the end of a cycle).
    assert!(
        evolver.population_size() <= max_pop + 1,
        "population ({}) should be <= max_population + 1 ({})",
        evolver.population_size(),
        max_pop + 1
    );
}
