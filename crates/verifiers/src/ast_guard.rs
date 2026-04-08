//! AST-based guard that uses the effect classifier to block dangerous commands.
//!
//! This guard analyses `Bash` tool calls through the tree-sitter-bash AST
//! (via [`crate::effect_classifier`]) and assigns a safety score based on the
//! combination of effects detected.  Non-Bash tools pass through with score 100.
//!
//! Feature-gated behind `ast-guard`.

use crate::effect_classifier::{classify_command, CommandEffect, EffectKind};
use crate::guard_pipeline::{Guard, GuardResult};

/// Guard that classifies shell commands via AST-based effect analysis.
pub struct AstGuard;

impl Guard for AstGuard {
    fn name(&self) -> &str {
        "ast"
    }

    fn check(&self, tool_name: &str, input: &str) -> GuardResult {
        // Only analyse Bash tool calls; everything else passes through.
        if !tool_name.eq_ignore_ascii_case("bash") {
            return GuardResult {
                guard_name: self.name().to_string(),
                score: 100,
                allowed: true,
                reason: None,
            };
        }

        let effects = classify_command(input);

        // If the classifier returned nothing, the command is unknown.
        if effects.is_empty() {
            return GuardResult {
                guard_name: self.name().to_string(),
                score: 100,
                allowed: true,
                reason: None,
            };
        }

        // Apply scoring rules in priority order (lowest score wins).
        let (score, reason) = score_effects(&effects);

        let allowed = score >= 50;

        GuardResult {
            guard_name: self.name().to_string(),
            score,
            allowed,
            reason,
        }
    }
}

/// Score a set of effects and return `(score, optional_reason)`.
///
/// Rules are evaluated in priority order; the first matching rule wins.
fn score_effects(effects: &[CommandEffect]) -> (u8, Option<String>) {
    // Rule 1: Delete + targets_root  -> 0 DENY
    for eff in effects {
        if eff.kind == EffectKind::Delete && eff.targets_root {
            return (
                0,
                Some(format!(
                    "delete targeting root via `{}`",
                    eff.command
                )),
            );
        }
    }

    // Rule 2: Tainted execution  -> 10 ASK
    for eff in effects {
        if eff.tainted && eff.kind == EffectKind::Execute {
            return (
                10,
                Some(format!(
                    "tainted execution: `{}` receives untrusted piped input",
                    eff.command
                )),
            );
        }
    }

    // Rule 3: Delete (non-root)  -> 20 ASK
    for eff in effects {
        if eff.kind == EffectKind::Delete && !eff.targets_root {
            return (
                20,
                Some(format!("delete operation via `{}`", eff.command)),
            );
        }
    }

    // Rule 4: Privilege escalation  -> 30 ASK
    for eff in effects {
        if eff.kind == EffectKind::Privilege {
            return (
                30,
                Some(format!(
                    "privilege escalation via `{}`",
                    eff.command
                )),
            );
        }
    }

    // Rule 5: Network + Write combo  -> 30 ASK
    let has_network = effects.iter().any(|e| e.kind == EffectKind::Network);
    let has_write = effects.iter().any(|e| e.kind == EffectKind::Write);
    if has_network && has_write {
        return (
            30,
            Some("network command combined with write operation".to_string()),
        );
    }

    // Rule 6: Unknown command  -> 50 ALLOW (with note)
    let all_unknown = effects.iter().all(|e| e.kind == EffectKind::Unknown);
    if all_unknown {
        let cmds: Vec<&str> = effects.iter().map(|e| e.command.as_str()).collect();
        return (
            50,
            Some(format!("unknown command(s): {}", cmds.join(", "))),
        );
    }

    // Rule 7: Read-only / safe  -> 100 ALLOW
    (100, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_bash_passes_through() {
        let guard = AstGuard;
        let result = guard.check("Read", "anything");
        assert!(result.allowed);
        assert_eq!(result.score, 100);
    }

    #[test]
    fn empty_input_passes() {
        let guard = AstGuard;
        let result = guard.check("Bash", "");
        assert!(result.allowed);
        assert_eq!(result.score, 100);
    }
}
