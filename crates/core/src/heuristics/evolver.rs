//! Evolutionary heuristic optimiser — Layer 0 meta-metacognition.
//!
//! Maintains a *population* of [`HeuristicVersion`]s, each containing a set of
//! risk-weight and strategy-preference parameters. After each batch of sessions,
//! the evolver:
//!
//! 1. **Evaluates** every version's fitness against recent [`SessionOutcome`]s.
//! 2. **Selects** the top performers (tournament selection, capped at `max_population`).
//! 3. **Mutates** the best version to produce a new candidate.
//!
//! Mutations are purely **statistical** (OPENSAGE-style), not LLM-driven:
//! - Adjust a random `risk_weight` by +/-10-20 %
//! - Swap the preferred strategy for a random `(risk, difficulty)` pair
//! - Add or remove a risk marker

use std::collections::HashMap;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::fitness::{FitnessScore, SessionOutcome};

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// Maximum number of recent outcomes retained for fitness evaluation.
const MAX_OUTCOMES: usize = 20;

/// Maximum expected tokens per session (used for normalisation).
const MAX_EXPECTED_TOKENS: u64 = 100_000;

/// Maximum expected duration per session in ms (used for normalisation).
const MAX_EXPECTED_DURATION: u64 = 300_000; // 5 minutes

/// Available risk markers that can be added during mutation.
const RISK_MARKERS: &[&str] = &[
    "fs_write",
    "exec_command",
    "network_access",
    "env_mutation",
    "credential_access",
    "large_diff",
    "multi_file",
    "untested_path",
];

// ──────────────────────────────────────────────────────────────
// HeuristicVersion
// ──────────────────────────────────────────────────────────────

/// A versioned set of heuristic parameters.
///
/// Each version carries its own risk-weight map and strategy-preference map.
/// Generations increase monotonically; `parent_id` tracks lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicVersion {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// Generation number (monotonically increasing).
    pub generation: u32,
    /// ID of the parent version this was mutated from (`None` for seed).
    pub parent_id: Option<String>,
    /// Multi-objective fitness score.
    pub fitness: FitnessScore,
    /// Risk marker -> severity weight (higher = more severe).
    pub risk_weights: HashMap<String, f64>,
    /// `"(risk_level,difficulty)"` -> preferred strategy name.
    pub strategy_scores: HashMap<String, f64>,
    /// ISO-8601 timestamp of creation.
    pub created_at: String,
}

impl HeuristicVersion {
    /// Create a new seed version with default heuristic parameters.
    pub fn seed() -> Self {
        let mut risk_weights = HashMap::new();
        risk_weights.insert("fs_write".into(), 0.6);
        risk_weights.insert("exec_command".into(), 0.8);
        risk_weights.insert("network_access".into(), 0.5);
        risk_weights.insert("env_mutation".into(), 0.7);
        risk_weights.insert("credential_access".into(), 0.9);
        risk_weights.insert("large_diff".into(), 0.4);
        risk_weights.insert("multi_file".into(), 0.3);
        risk_weights.insert("untested_path".into(), 0.5);

        let mut strategy_scores = HashMap::new();
        // Keys are "(risk_level,difficulty)" pairs; values are strategy
        // preference scores where higher means more preferred.
        strategy_scores.insert("(low,easy)".into(), 0.2); // prefers single
        strategy_scores.insert("(low,medium)".into(), 0.4);
        strategy_scores.insert("(medium,easy)".into(), 0.5);
        strategy_scores.insert("(medium,medium)".into(), 0.6);
        strategy_scores.insert("(medium,hard)".into(), 0.7);
        strategy_scores.insert("(high,easy)".into(), 0.7);
        strategy_scores.insert("(high,medium)".into(), 0.8);
        strategy_scores.insert("(high,hard)".into(), 0.9);

        Self {
            id: Uuid::new_v4().to_string(),
            generation: 0,
            parent_id: None,
            fitness: FitnessScore::zero(),
            risk_weights,
            strategy_scores,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ──────────────────────────────────────────────────────────────
// HeuristicEvolver
// ──────────────────────────────────────────────────────────────

/// Evolves heuristics based on session outcomes using statistical learning.
///
/// The evolver maintains a bounded population of [`HeuristicVersion`]s and a
/// rolling window of recent [`SessionOutcome`]s. Each evolution cycle evaluates
/// all versions, selects the fittest, and produces a mutated child.
pub struct HeuristicEvolver {
    /// Current population of heuristic versions.
    population: Vec<HeuristicVersion>,
    /// Upper bound on population size (default: 20).
    max_population: usize,
    /// Recent outcomes for fitness evaluation (capped at [`MAX_OUTCOMES`]).
    outcomes: Vec<SessionOutcome>,
}

impl HeuristicEvolver {
    /// Create a new evolver with the given population cap.
    ///
    /// A seed heuristic version is automatically inserted into the population.
    pub fn new(max_population: usize) -> Self {
        let seed = HeuristicVersion::seed();
        Self {
            population: vec![seed],
            max_population,
            outcomes: Vec::new(),
        }
    }

    /// Restore a previously persisted version into the population.
    /// Used at daemon startup to load state from SQLite.
    pub fn restore_version(&mut self, version: HeuristicVersion) {
        if self.population.len() < self.max_population {
            self.population.push(version);
        }
    }

    /// Record a session outcome for future fitness evaluation.
    ///
    /// Outcomes are kept in a rolling window of the most recent [`MAX_OUTCOMES`].
    pub fn record_outcome(&mut self, outcome: SessionOutcome) {
        self.outcomes.push(outcome);
        if self.outcomes.len() > MAX_OUTCOMES {
            self.outcomes.remove(0);
        }
    }

    /// Get the current best heuristic version (highest composite fitness).
    pub fn best(&self) -> Option<&HeuristicVersion> {
        self.population.iter().max_by(|a, b| {
            a.fitness
                .composite
                .partial_cmp(&b.fitness.composite)
                .unwrap()
        })
    }

    /// Create a mutated child from the best parent.
    ///
    /// Applies **one** random mutation:
    /// 1. Adjust a random `risk_weight` by +/-10-20 %.
    /// 2. Adjust a random `strategy_score` by +/-10-20 %.
    /// 3. Add a new risk marker or remove an existing one.
    ///
    /// Returns `None` if the population is empty.
    pub fn mutate_best(&mut self) -> Option<HeuristicVersion> {
        let parent = self.best()?.clone();
        let mut child = parent.clone();

        child.id = Uuid::new_v4().to_string();
        child.generation = parent.generation + 1;
        child.parent_id = Some(parent.id.clone());
        child.created_at = chrono::Utc::now().to_rfc3339();

        let mut rng = rand::thread_rng();
        let mutation_type = rng.gen_range(0..3);

        match mutation_type {
            0 => {
                // Mutation 1: Adjust a random risk_weight by +/-10-20%.
                if !child.risk_weights.is_empty() {
                    let keys: Vec<String> = child.risk_weights.keys().cloned().collect();
                    let key = &keys[rng.gen_range(0..keys.len())];
                    if let Some(weight) = child.risk_weights.get_mut(key) {
                        let factor = if rng.gen_bool(0.5) {
                            1.0 + rng.gen_range(0.10..=0.20)
                        } else {
                            1.0 - rng.gen_range(0.10..=0.20)
                        };
                        *weight = (*weight * factor).clamp(0.0, 1.0);
                    }
                }
            }
            1 => {
                // Mutation 2: Adjust a random strategy_score by +/-10-20%.
                if !child.strategy_scores.is_empty() {
                    let keys: Vec<String> = child.strategy_scores.keys().cloned().collect();
                    let key = &keys[rng.gen_range(0..keys.len())];
                    if let Some(score) = child.strategy_scores.get_mut(key) {
                        let factor = if rng.gen_bool(0.5) {
                            1.0 + rng.gen_range(0.10..=0.20)
                        } else {
                            1.0 - rng.gen_range(0.10..=0.20)
                        };
                        *score = (*score * factor).clamp(0.0, 1.0);
                    }
                }
            }
            2 => {
                // Mutation 3: Add or remove a risk marker.
                if rng.gen_bool(0.5) && !child.risk_weights.is_empty() {
                    // Remove a random existing marker.
                    let keys: Vec<String> = child.risk_weights.keys().cloned().collect();
                    let key = &keys[rng.gen_range(0..keys.len())];
                    child.risk_weights.remove(key);
                } else {
                    // Add a new marker with a random weight.
                    let marker = RISK_MARKERS[rng.gen_range(0..RISK_MARKERS.len())];
                    child
                        .risk_weights
                        .entry(marker.into())
                        .or_insert_with(|| rng.gen_range(0.1..=0.9));
                }
            }
            _ => unreachable!(),
        }

        self.population.push(child.clone());
        Some(child)
    }

    /// Evaluate fitness of all versions against recorded outcomes.
    ///
    /// For each version the fitness is computed from the recent outcomes:
    /// - `success_rate` = successful outcomes / total outcomes
    /// - `token_efficiency` = 1.0 - (avg_tokens / MAX_EXPECTED_TOKENS)
    /// - `latency_score` = 1.0 - (avg_duration / MAX_EXPECTED_DURATION)
    ///
    /// Versions whose strategy preferences better match the successful outcomes
    /// receive a bonus to their success rate.
    pub fn evaluate_all(&mut self) {
        if self.outcomes.is_empty() {
            return;
        }

        let outcomes = &self.outcomes;
        let total = outcomes.len() as f64;

        for version in &mut self.population {
            // Base success rate from outcomes.
            let successes = outcomes.iter().filter(|o| o.success).count() as f64;
            let base_success_rate = successes / total;

            // Strategy alignment bonus: reward versions whose strategy_scores
            // align with the strategies that actually succeeded.
            let mut alignment_bonus = 0.0_f64;
            let mut alignment_count = 0_u32;
            for outcome in outcomes {
                let key = format!("({},medium)", outcome.risk_level);
                if let Some(&score) = version.strategy_scores.get(&key) {
                    if outcome.success {
                        // The strategy worked — reward versions that score this
                        // risk level highly (i.e. they would have chosen a
                        // more cautious strategy, matching the success).
                        alignment_bonus += score;
                    } else {
                        // The strategy failed — penalise.
                        alignment_bonus -= score * 0.5;
                    }
                    alignment_count += 1;
                }
            }
            let strategy_modifier = if alignment_count > 0 {
                (alignment_bonus / alignment_count as f64).clamp(-0.2, 0.2)
            } else {
                0.0
            };

            let success_rate = (base_success_rate + strategy_modifier).clamp(0.0, 1.0);

            // Token efficiency: 1.0 - (avg_tokens / max_expected).
            let avg_tokens = outcomes.iter().map(|o| o.tokens_consumed).sum::<u64>() as f64 / total;
            let token_efficiency = (1.0 - avg_tokens / MAX_EXPECTED_TOKENS as f64).clamp(0.0, 1.0);

            // Latency score: 1.0 - (avg_duration / max_expected).
            let avg_duration = outcomes.iter().map(|o| o.duration_ms).sum::<u64>() as f64 / total;
            let latency_score = (1.0 - avg_duration / MAX_EXPECTED_DURATION as f64).clamp(0.0, 1.0);

            version.fitness = FitnessScore::compute(success_rate, token_efficiency, latency_score);
        }
    }

    /// Run one evolution cycle: evaluate -> select -> mutate.
    ///
    /// 1. Evaluate all versions against recent outcomes.
    /// 2. Sort by composite fitness (descending).
    /// 3. Keep the top `max_population` versions (tournament selection).
    /// 4. Mutate the best to produce a new candidate.
    /// 5. Return the new best.
    pub fn evolve_generation(&mut self) -> Option<&HeuristicVersion> {
        // 1. Evaluate.
        self.evaluate_all();

        // 2. Sort by composite fitness, descending.
        self.population.sort_by(|a, b| {
            b.fitness
                .composite
                .partial_cmp(&a.fitness.composite)
                .unwrap()
        });

        // 3. Truncate to max_population (tournament selection — keep the best).
        if self.population.len() > self.max_population {
            self.population.truncate(self.max_population);
        }

        // 4. Mutate the best to add a new candidate.
        self.mutate_best();

        // 5. Return the current best.
        self.best()
    }

    /// Number of heuristic versions currently in the population.
    pub fn population_size(&self) -> usize {
        self.population.len()
    }

    /// Highest generation number in the current population.
    pub fn generation(&self) -> u32 {
        self.population
            .iter()
            .map(|v| v.generation)
            .max()
            .unwrap_or(0)
    }

    /// Get the full population (read-only).
    pub fn population(&self) -> &[HeuristicVersion] {
        &self.population
    }

    /// Get the recorded outcomes (read-only).
    pub fn outcomes(&self) -> &[SessionOutcome] {
        &self.outcomes
    }
}

// ──────────────────────────────────────────────────────────────
// Unit tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_has_default_weights() {
        let seed = HeuristicVersion::seed();
        assert_eq!(seed.generation, 0);
        assert!(seed.parent_id.is_none());
        assert!(!seed.risk_weights.is_empty());
        assert!(!seed.strategy_scores.is_empty());
    }

    #[test]
    fn new_evolver_has_seed() {
        let evolver = HeuristicEvolver::new(20);
        assert_eq!(evolver.population_size(), 1);
        assert_eq!(evolver.generation(), 0);
    }
}
