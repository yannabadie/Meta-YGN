use std::collections::HashSet;

use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 10: Memory compaction with semantic lesson clustering.
///
/// Groups lessons by word overlap (Jaccard similarity > 0.5 on non-trivial
/// words), merges duplicates with a count suffix like "(x3)", and keeps at
/// most `MAX_CLUSTERS` unique clusters.  Appends a compact summary line
/// for downstream consumption and archival.
pub struct CompactStage;

const MAX_CLUSTERS: usize = 10;

/// Cluster lessons by word overlap.
///
/// Two lessons that share >50% Jaccard similarity on non-trivial words
/// (length > 2) are merged into a single cluster.  The representative
/// text is the first lesson that founded the cluster; subsequent merges
/// only bump the count.
///
/// Returns at most `max_clusters` entries, sorted by frequency (descending).
/// Merged entries carry a `" (xN)"` suffix.
pub fn cluster_lessons(lessons: &[String], max_clusters: usize) -> Vec<String> {
    if lessons.is_empty() {
        return Vec::new();
    }

    let mut clusters: Vec<(String, u32)> = Vec::new(); // (representative, count)

    for lesson in lessons {
        let lesson_words: HashSet<&str> = lesson
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        // Find a matching cluster (>50% Jaccard word overlap)
        let mut merged = false;
        for (rep, count) in &mut clusters {
            let rep_words: HashSet<&str> = rep
                .split_whitespace()
                .filter(|w| w.len() > 2)
                .collect();

            if rep_words.is_empty() || lesson_words.is_empty() {
                continue;
            }

            let intersection = lesson_words.intersection(&rep_words).count();
            let union = lesson_words.union(&rep_words).count();
            let overlap = intersection as f64 / union as f64;

            if overlap > 0.5 {
                *count += 1;
                merged = true;
                break;
            }
        }

        if !merged {
            clusters.push((lesson.clone(), 1));
        }
    }

    // Sort by count descending, take max_clusters
    clusters.sort_by(|a, b| b.1.cmp(&a.1));
    clusters.truncate(max_clusters);

    clusters
        .into_iter()
        .map(|(rep, count)| {
            if count > 1 {
                format!("{} (x{})", rep, count)
            } else {
                rep
            }
        })
        .collect()
}

impl Stage for CompactStage {
    fn name(&self) -> &'static str {
        "compact"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // 1. Cluster lessons semantically
        ctx.lessons = cluster_lessons(&ctx.lessons, MAX_CLUSTERS);

        // 2. Generate compact summary
        let summary = format!(
            "[compact] {} lessons, {} verifications, quality={:.2}",
            ctx.lessons.len(),
            ctx.verification_results.len(),
            ctx.metacog_vector.overall_quality()
        );
        ctx.lessons.push(summary);

        tracing::debug!(
            stage = self.name(),
            lesson_count = ctx.lessons.len(),
            verification_count = ctx.verification_results.len(),
            "compacted context"
        );

        StageResult::Continue
    }
}
