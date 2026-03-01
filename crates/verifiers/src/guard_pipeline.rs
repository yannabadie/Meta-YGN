use regex::Regex;

/// Result from a single guard check.
#[derive(Debug, Clone)]
pub struct GuardResult {
    pub guard_name: String,
    pub score: u8,
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Aggregate decision from the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineDecision {
    pub allowed: bool,
    pub results: Vec<GuardResult>,
    pub aggregate_score: u8,
    pub blocking_guard: Option<String>,
}

/// Trait all guards implement.
pub trait Guard: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, tool_name: &str, input: &str) -> GuardResult;
}

// ---------------------------------------------------------------------------
// DestructiveGuard
// ---------------------------------------------------------------------------

/// Blocks destructive shell patterns such as `rm -rf /`, `mkfs`, `dd if=`,
/// `shutdown`, `reboot`, fork bombs, and `chmod 777 /`.
pub struct DestructiveGuard;

impl DestructiveGuard {
    fn patterns() -> Vec<Regex> {
        [
            r"rm\s+-rf\s+/",
            r"sudo\s+rm\s+-rf",
            r"\bmkfs\b",
            r"\bdd\s+if=",
            r"\bshutdown\b",
            r"\breboot\b",
            r":\(\)\{.*\|.*\}", // fork bomb
            r"chmod\s+777\s+/",
        ]
        .iter()
        .map(|p| Regex::new(p).expect("invalid destructive pattern"))
        .collect()
    }
}

impl Guard for DestructiveGuard {
    fn name(&self) -> &str {
        "destructive"
    }

    fn check(&self, _tool_name: &str, input: &str) -> GuardResult {
        let patterns = Self::patterns();
        for pat in &patterns {
            if pat.is_match(input) {
                return GuardResult {
                    guard_name: self.name().to_string(),
                    score: 0,
                    allowed: false,
                    reason: Some(format!("Destructive pattern detected: {}", pat.as_str())),
                };
            }
        }
        GuardResult {
            guard_name: self.name().to_string(),
            score: 100,
            allowed: true,
            reason: None,
        }
    }
}

// ---------------------------------------------------------------------------
// HighRiskGuard
// ---------------------------------------------------------------------------

/// Blocks high-risk operations that typically require confirmation, such as
/// `git push`, `terraform apply`, `kubectl delete`, and piped installs.
pub struct HighRiskGuard;

impl HighRiskGuard {
    fn patterns() -> Vec<Regex> {
        [
            r"\bgit\s+push\b",
            r"\bgit\s+reset\s+--hard\b",
            r"\bterraform\s+apply\b",
            r"\bterraform\s+destroy\b",
            r"\bkubectl\s+apply\b",
            r"\bkubectl\s+delete\b",
            r"\bcurl\b.*\|\s*bash",
            r"\bsudo\b",
            r"\bdocker\s+push\b",
            r"\bdocker\s+prune\b",
        ]
        .iter()
        .map(|p| Regex::new(p).expect("invalid high-risk pattern"))
        .collect()
    }
}

impl Guard for HighRiskGuard {
    fn name(&self) -> &str {
        "high_risk"
    }

    fn check(&self, _tool_name: &str, input: &str) -> GuardResult {
        let patterns = Self::patterns();
        for pat in &patterns {
            if pat.is_match(input) {
                return GuardResult {
                    guard_name: self.name().to_string(),
                    score: 30,
                    allowed: false,
                    reason: Some(format!("High-risk operation detected: {}", pat.as_str())),
                };
            }
        }
        GuardResult {
            guard_name: self.name().to_string(),
            score: 100,
            allowed: true,
            reason: None,
        }
    }
}

// ---------------------------------------------------------------------------
// SecretPathGuard
// ---------------------------------------------------------------------------

/// Blocks commands that reference paths containing secrets such as `.env`,
/// `*.pem`, `id_rsa`, and `credentials.json`.
pub struct SecretPathGuard;

impl SecretPathGuard {
    fn patterns() -> Vec<Regex> {
        [
            r"\.env\b",
            r"\bsecrets/",
            r"\.pem\b",
            r"\.key\b",
            r"\bid_rsa\b",
            r"\bcredentials\.json\b",
            r"\.npmrc\b",
            r"\.pypirc\b",
            r"\bkubeconfig\b",
        ]
        .iter()
        .map(|p| Regex::new(p).expect("invalid secret-path pattern"))
        .collect()
    }
}

impl Guard for SecretPathGuard {
    fn name(&self) -> &str {
        "secret_path"
    }

    fn check(&self, _tool_name: &str, input: &str) -> GuardResult {
        let patterns = Self::patterns();
        for pat in &patterns {
            if pat.is_match(input) {
                return GuardResult {
                    guard_name: self.name().to_string(),
                    score: 20,
                    allowed: false,
                    reason: Some(format!("Secret path detected: {}", pat.as_str())),
                };
            }
        }
        GuardResult {
            guard_name: self.name().to_string(),
            score: 100,
            allowed: true,
            reason: None,
        }
    }
}

// ---------------------------------------------------------------------------
// McpGuard
// ---------------------------------------------------------------------------

/// Blocks any tool call whose name starts with `mcp__`.
pub struct McpGuard;

impl Guard for McpGuard {
    fn name(&self) -> &str {
        "mcp"
    }

    fn check(&self, tool_name: &str, _input: &str) -> GuardResult {
        if tool_name.starts_with("mcp__") {
            return GuardResult {
                guard_name: self.name().to_string(),
                score: 40,
                allowed: false,
                reason: Some(format!("MCP tool call gated: {tool_name}")),
            };
        }
        GuardResult {
            guard_name: self.name().to_string(),
            score: 100,
            allowed: true,
            reason: None,
        }
    }
}

// ---------------------------------------------------------------------------
// DefaultGuard
// ---------------------------------------------------------------------------

/// Always allows — produces score 100.
pub struct DefaultGuard;

impl Guard for DefaultGuard {
    fn name(&self) -> &str {
        "default"
    }

    fn check(&self, _tool_name: &str, _input: &str) -> GuardResult {
        GuardResult {
            guard_name: self.name().to_string(),
            score: 100,
            allowed: true,
            reason: None,
        }
    }
}

// ---------------------------------------------------------------------------
// GuardPipeline
// ---------------------------------------------------------------------------

/// Composable pipeline that runs multiple guards and produces an aggregate
/// decision.  The aggregate score is the **minimum** across all guard scores.
/// If any guard blocks, the pipeline blocks and records the first blocking
/// guard encountered.
pub struct GuardPipeline {
    guards: Vec<Box<dyn Guard>>,
}

impl GuardPipeline {
    /// Creates a pipeline with the default guard ordering.
    pub fn new() -> Self {
        Self {
            guards: vec![
                Box::new(DestructiveGuard),
                Box::new(HighRiskGuard),
                Box::new(SecretPathGuard),
                Box::new(McpGuard),
                Box::new(DefaultGuard),
            ],
        }
    }

    /// Creates a pipeline with a custom set of guards.
    pub fn with_guards(guards: Vec<Box<dyn Guard>>) -> Self {
        Self { guards }
    }

    /// Runs every guard and returns a [`PipelineDecision`].
    pub fn check(&self, tool_name: &str, input: &str) -> PipelineDecision {
        let mut results = Vec::with_capacity(self.guards.len());
        let mut aggregate_score: u8 = 100;
        let mut allowed = true;
        let mut blocking_guard: Option<String> = None;

        for guard in &self.guards {
            let result = guard.check(tool_name, input);

            if result.score < aggregate_score {
                aggregate_score = result.score;
            }

            if !result.allowed && blocking_guard.is_none() {
                allowed = false;
                blocking_guard = Some(result.guard_name.clone());
            }

            // Even if we already found a blocker, keep running to collect
            // all guard results for observability.
            if !result.allowed {
                allowed = false;
            }

            results.push(result);
        }

        PipelineDecision {
            allowed,
            results,
            aggregate_score,
            blocking_guard,
        }
    }
}

impl Default for GuardPipeline {
    fn default() -> Self {
        Self::new()
    }
}
