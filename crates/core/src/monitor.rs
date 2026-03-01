//! MASC (Metacognitive Anomaly via Similarity of Context) anomaly detector.
//!
//! Detects when the AI's current reasoning step is anomalous compared to its
//! historical patterns using TF-IDF cosine similarity. Adapted from NEXUS's
//! `metacognitive_monitor.py`.
//!
//! The monitor maintains a sliding window of recent reasoning steps. When a new
//! step arrives, it computes TF-IDF vectors and checks cosine similarity against
//! the window. If similarity drops below a threshold it flags an anomaly; if it
//! exceeds an upper threshold it flags stagnation (repetitive reasoning).

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// A reasoning step to monitor.
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub content: String,
    pub step_number: usize,
    pub timestamp: DateTime<Utc>,
}

/// Result of anomaly detection.
#[derive(Debug, Clone, Serialize)]
pub struct AnomalyReport {
    /// Whether the step was flagged as anomalous (too different **or** too
    /// similar / stagnant).
    pub is_anomalous: bool,
    /// Average cosine similarity against the history window.
    /// `0.0` = completely different, `1.0` = identical.
    pub similarity_score: f64,
    /// The threshold that was applied (anomaly or stagnation).
    pub threshold: f64,
    /// Human-readable explanation when an anomaly is detected.
    pub reason: Option<String>,
}

/// Configuration for [`MascMonitor`].
pub struct MonitorConfig {
    /// How many recent steps to keep in the comparison window.
    pub window_size: usize,
    /// Similarity **below** this value flags an anomaly (off-track reasoning).
    pub anomaly_threshold: f64,
    /// Similarity **above** this value flags stagnation (repetitive reasoning).
    pub stagnation_threshold: f64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            window_size: 10,
            anomaly_threshold: 0.15,
            stagnation_threshold: 0.95,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// MascMonitor
// ──────────────────────────────────────────────────────────────

/// Metacognitive anomaly detector backed by TF-IDF cosine similarity.
pub struct MascMonitor {
    config: MonitorConfig,
    history: Vec<ReasoningStep>,
    /// Global document-frequency map: term -> number of documents containing it.
    vocabulary: HashMap<String, usize>,
}

impl MascMonitor {
    /// Create a new monitor with explicit configuration.
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            vocabulary: HashMap::new(),
        }
    }

    /// Create a new monitor with default thresholds.
    pub fn with_defaults() -> Self {
        Self::new(MonitorConfig::default())
    }

    /// Record a new step **and** check for anomalies.
    ///
    /// 1. Tokenize the new step's content.
    /// 2. Update vocabulary (document frequencies).
    /// 3. Compute TF-IDF vector for the new step.
    /// 4. Compute average cosine similarity against the history window.
    /// 5. If similarity < `anomaly_threshold` -> anomaly (off-track).
    /// 6. If similarity > `stagnation_threshold` -> stagnation (repeating).
    /// 7. Append step to history (sliding window: drop oldest if > window_size).
    /// 8. Return [`AnomalyReport`].
    pub fn observe(&mut self, step: ReasoningStep) -> AnomalyReport {
        let tokens = tokenize(&step.content);

        // Update document frequencies with the new step's unique terms.
        let unique_terms: std::collections::HashSet<&String> = tokens.iter().collect();
        for term in &unique_terms {
            *self.vocabulary.entry((*term).clone()).or_insert(0) += 1;
        }

        // If history is empty, this is the first step -- never anomalous.
        if self.history.is_empty() {
            self.history.push(step);
            return AnomalyReport {
                is_anomalous: false,
                similarity_score: 1.0,
                threshold: self.config.anomaly_threshold,
                reason: None,
            };
        }

        // Total documents = existing history + the new step.
        let doc_count = self.history.len() + 1;

        // TF-IDF vector for the incoming step.
        let new_vec = tfidf_vector(&tokens, doc_count, &self.vocabulary);

        // Compute average cosine similarity against all history steps.
        let mut total_sim = 0.0;
        for past in &self.history {
            let past_tokens = tokenize(&past.content);
            let past_vec = tfidf_vector(&past_tokens, doc_count, &self.vocabulary);
            total_sim += cosine_similarity(&new_vec, &past_vec);
        }
        let avg_sim = total_sim / self.history.len() as f64;

        // Classify.
        let report = if avg_sim < self.config.anomaly_threshold {
            AnomalyReport {
                is_anomalous: true,
                similarity_score: avg_sim,
                threshold: self.config.anomaly_threshold,
                reason: Some(format!(
                    "Reasoning diverged from recent context (similarity {:.3} < threshold {:.3})",
                    avg_sim, self.config.anomaly_threshold,
                )),
            }
        } else if avg_sim > self.config.stagnation_threshold {
            AnomalyReport {
                is_anomalous: true,
                similarity_score: avg_sim,
                threshold: self.config.stagnation_threshold,
                reason: Some(format!(
                    "Reasoning appears stagnant/repetitive (similarity {:.3} > threshold {:.3})",
                    avg_sim, self.config.stagnation_threshold,
                )),
            }
        } else {
            AnomalyReport {
                is_anomalous: false,
                similarity_score: avg_sim,
                threshold: self.config.anomaly_threshold,
                reason: None,
            }
        };

        // Maintain sliding window.
        self.history.push(step);
        if self.history.len() > self.config.window_size {
            self.history.remove(0);
        }

        report
    }

    /// Check if `content` would be anomalous against current history **without
    /// recording** it.
    pub fn check(&self, content: &str) -> AnomalyReport {
        if self.history.is_empty() {
            return AnomalyReport {
                is_anomalous: false,
                similarity_score: 1.0,
                threshold: self.config.anomaly_threshold,
                reason: None,
            };
        }

        let tokens = tokenize(content);

        // Simulate updated vocabulary with the new document.
        let mut vocab = self.vocabulary.clone();
        let unique_terms: std::collections::HashSet<&String> = tokens.iter().collect();
        for term in &unique_terms {
            *vocab.entry((*term).clone()).or_insert(0) += 1;
        }

        let doc_count = self.history.len() + 1;
        let new_vec = tfidf_vector(&tokens, doc_count, &vocab);

        let mut total_sim = 0.0;
        for past in &self.history {
            let past_tokens = tokenize(&past.content);
            let past_vec = tfidf_vector(&past_tokens, doc_count, &vocab);
            total_sim += cosine_similarity(&new_vec, &past_vec);
        }
        let avg_sim = total_sim / self.history.len() as f64;

        if avg_sim < self.config.anomaly_threshold {
            AnomalyReport {
                is_anomalous: true,
                similarity_score: avg_sim,
                threshold: self.config.anomaly_threshold,
                reason: Some(format!(
                    "Reasoning diverged from recent context (similarity {:.3} < threshold {:.3})",
                    avg_sim, self.config.anomaly_threshold,
                )),
            }
        } else if avg_sim > self.config.stagnation_threshold {
            AnomalyReport {
                is_anomalous: true,
                similarity_score: avg_sim,
                threshold: self.config.stagnation_threshold,
                reason: Some(format!(
                    "Reasoning appears stagnant/repetitive (similarity {:.3} > threshold {:.3})",
                    avg_sim, self.config.stagnation_threshold,
                )),
            }
        } else {
            AnomalyReport {
                is_anomalous: false,
                similarity_score: avg_sim,
                threshold: self.config.anomaly_threshold,
                reason: None,
            }
        }
    }

    /// Get the current history window.
    pub fn history(&self) -> &[ReasoningStep] {
        &self.history
    }

    /// Clear all history and vocabulary.
    pub fn reset(&mut self) {
        self.history.clear();
        self.vocabulary.clear();
    }
}

// ──────────────────────────────────────────────────────────────
// TF-IDF primitives
// ──────────────────────────────────────────────────────────────

/// Tokenize text into lowercase alphanumeric words.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Compute raw term frequency for a token list.
/// TF(t,d) = count(t in d) / |d|
fn term_frequency(tokens: &[String]) -> HashMap<String, f64> {
    let mut counts: HashMap<String, f64> = HashMap::new();
    for t in tokens {
        *counts.entry(t.clone()).or_insert(0.0) += 1.0;
    }
    let len = tokens.len() as f64;
    if len > 0.0 {
        for v in counts.values_mut() {
            *v /= len;
        }
    }
    counts
}

/// Inverse document frequency for a single term.
/// IDF(t) = ln(1 + N / (1 + df(t)))   (smoothed, always non-negative)
fn inverse_document_frequency(term: &str, doc_count: usize, df: &HashMap<String, usize>) -> f64 {
    let n = doc_count as f64;
    let df_t = df.get(term).copied().unwrap_or(0) as f64;
    (1.0 + n / (1.0 + df_t)).ln()
}

/// Build a TF-IDF vector (sparse, stored as `HashMap`) for one document.
fn tfidf_vector(
    tokens: &[String],
    doc_count: usize,
    df: &HashMap<String, usize>,
) -> HashMap<String, f64> {
    let tf = term_frequency(tokens);
    let mut vec = HashMap::new();
    for (term, tf_val) in &tf {
        let idf = inverse_document_frequency(term, doc_count, df);
        let score = tf_val * idf;
        if score != 0.0 {
            vec.insert(term.clone(), score);
        }
    }
    vec
}

/// Cosine similarity between two sparse TF-IDF vectors.
/// Returns `0.0` when either vector is zero-length.
pub fn cosine_similarity(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let dot: f64 = a
        .iter()
        .filter_map(|(k, va)| b.get(k).map(|vb| va * vb))
        .sum();

    let mag_a: f64 = a.values().map(|v| v * v).sum::<f64>().sqrt();
    let mag_b: f64 = b.values().map(|v| v * v).sum::<f64>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

// ──────────────────────────────────────────────────────────────
// Unit tests (internal)
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        let tokens = tokenize("Hello, World! 123");
        assert_eq!(tokens, vec!["hello", "world", "123"]);
    }

    #[test]
    fn term_frequency_counts() {
        let tokens = vec!["a".into(), "b".into(), "a".into()];
        let tf = term_frequency(&tokens);
        assert!((tf["a"] - 2.0 / 3.0).abs() < 1e-9);
        assert!((tf["b"] - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn cosine_identical_vectors() {
        let mut a = HashMap::new();
        a.insert("x".into(), 1.0);
        a.insert("y".into(), 2.0);
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let mut a = HashMap::new();
        a.insert("x".into(), 1.0);
        let mut b = HashMap::new();
        b.insert("y".into(), 1.0);
        let sim = cosine_similarity(&a, &b);
        assert!((sim).abs() < 1e-9);
    }
}
