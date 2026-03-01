use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A detected pattern of tool usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPattern {
    /// Ordered sequence of tool names, e.g. `["Grep", "Read", "Edit"]`.
    pub tools: Vec<String>,
    /// How many times this exact sequence has been observed.
    pub count: u32,
    /// ISO 8601 timestamp of the most recent observation.
    pub last_seen: String,
    /// SHA-256 hex digest of the tool sequence (dedup key).
    pub hash: String,
}

/// Observes tool sequences and crystallizes recurring patterns into skill
/// templates once they exceed an observation threshold.
pub struct SkillCrystallizer {
    patterns: HashMap<String, ToolPattern>,
    threshold: u32,
}

impl SkillCrystallizer {
    pub fn new(threshold: u32) -> Self {
        Self {
            patterns: HashMap::new(),
            threshold,
        }
    }

    /// Record an observed tool sequence.
    pub fn observe(&mut self, tools: &[String]) {
        if tools.is_empty() {
            return;
        }
        let hash = Self::hash_sequence(tools);
        let entry = self
            .patterns
            .entry(hash.clone())
            .or_insert_with(|| ToolPattern {
                tools: tools.to_vec(),
                count: 0,
                last_seen: String::new(),
                hash,
            });
        entry.count += 1;
        entry.last_seen = chrono::Utc::now().to_rfc3339();
    }

    /// Get all patterns that meet the crystallization threshold.
    pub fn crystallized(&self) -> Vec<&ToolPattern> {
        self.patterns
            .values()
            .filter(|p| p.count >= self.threshold)
            .collect()
    }

    /// Generate a SKILL.md template for a crystallized pattern.
    pub fn generate_skill_md(pattern: &ToolPattern) -> String {
        let tools_list = pattern.tools.join(" -> ");
        let short_hash = &pattern.hash[..8];
        let tools_numbered = pattern
            .tools
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "---\n\
             name: crystallized-{short_hash}\n\
             description: Auto-detected pattern ({count}x): {tools}\n\
             user-invocable: true\n\
             ---\n\
             \n\
             # Crystallized Pattern\n\
             \n\
             This workflow was automatically detected from {count} observations.\n\
             \n\
             ## Tool Sequence\n\
             {tools_numbered}\n\
             \n\
             ## Usage\n\
             Invoke this skill when you need to perform the same sequence of operations.\n",
            short_hash = short_hash,
            count = pattern.count,
            tools = tools_list,
            tools_numbered = tools_numbered,
        )
    }

    /// Total patterns observed (including those below threshold).
    pub fn total_patterns(&self) -> usize {
        self.patterns.len()
    }

    /// Compute the SHA-256 hex digest of a tool sequence.
    fn hash_sequence(tools: &[String]) -> String {
        let mut hasher = Sha256::new();
        for tool in tools {
            hasher.update(tool.as_bytes());
            hasher.update(b"|");
        }
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
