import type { HookInput, HookOutput } from "@metaygn/shared/src/types.js";

/**
 * Destructive patterns — commands that can cause irreversible data loss.
 * Matched against the stringified tool_input. Deny immediately.
 */
const DESTRUCTIVE_PATTERNS: RegExp[] = [
  /\brm\s+-rf\s+\/(?!\w)/,       // rm -rf / (root wipe)
  /\bsudo\s+rm\s+-rf\b/,         // sudo rm -rf anything
  /\bmkfs\b/,                     // format filesystem
  /\bdd\s+if=/,                   // raw disk write
  /\bshutdown\b/,                 // shutdown system
  /\breboot\b/,                   // reboot system
];

/**
 * High-risk patterns — commands that deserve user confirmation.
 * Matched against the stringified tool_input. Ask for confirmation.
 */
const HIGH_RISK_PATTERNS: RegExp[] = [
  /\bgit\s+push\b/,              // pushing to remote
  /\bgit\s+reset\s+--hard\b/,    // destructive git reset
  /\bterraform\s+apply\b/,       // infra provisioning
  /\bterraform\s+destroy\b/,     // infra teardown
  /\bkubectl\s+apply\b/,         // k8s apply
  /\bkubectl\s+delete\b/,        // k8s delete
  /\bcurl\b.*\|\s*bash\b/,       // piping curl to bash
  /\bsudo\b/,                    // privilege escalation
];

/**
 * Check if a tool name is an MCP tool (external integration).
 * MCP tools start with "mcp__" and should require confirmation.
 */
function isMcpTool(toolName: string): boolean {
  return toolName.startsWith("mcp__");
}

/**
 * Serialize tool_input to a single string for pattern matching.
 */
function serializeInput(input: Record<string, unknown>): string {
  return Object.values(input)
    .map((v) => (typeof v === "string" ? v : JSON.stringify(v)))
    .join(" ");
}

/**
 * Local heuristic fallback that mirrors the daemon's security patterns.
 * Returns a HookOutput with deny/ask decision, or null to allow.
 */
export function evaluateFallback(input: HookInput): HookOutput | null {
  const toolName = input.tool_name ?? "";
  const toolInput = input.tool_input ?? {};
  const serialized = serializeInput(toolInput);

  // Gate MCP tools — require user confirmation
  if (isMcpTool(toolName)) {
    return {
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "ask",
        permissionDecisionReason: `MCP tool "${toolName}" requires user confirmation`,
      },
    };
  }

  // Check destructive patterns — deny outright
  for (const pattern of DESTRUCTIVE_PATTERNS) {
    if (pattern.test(serialized)) {
      return {
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "deny",
          permissionDecisionReason: `Blocked destructive pattern: ${pattern.source}`,
        },
      };
    }
  }

  // Check high-risk patterns — ask for confirmation
  for (const pattern of HIGH_RISK_PATTERNS) {
    if (pattern.test(serialized)) {
      return {
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "ask",
          permissionDecisionReason: `High-risk command detected: ${pattern.source}`,
        },
      };
    }
  }

  // No match — allow (return null)
  return null;
}

/**
 * Local fallback for user-prompt-submit: classify risk using keyword matching.
 * Returns a HookOutput with risk level and strategy hint.
 */
export function fallbackUserPromptSubmit(input: HookInput): HookOutput | null {
  const prompt = (input.prompt || "").toLowerCase();
  const highRisk = ["auth", "oauth", "token", "secret", "deploy", "payment", "database", "prod", "security", "delete", "terraform", "kubernetes"];
  const lowRisk = ["typo", "rename", "comment", "docs", "readme", "format", "lint", "cleanup"];

  const risk = highRisk.some(k => prompt.includes(k)) ? "HIGH"
    : lowRisk.some(k => prompt.includes(k)) ? "LOW" : "MEDIUM";

  return {
    hookSpecificOutput: {
      hookEventName: "UserPromptSubmit",
      additionalContext: `Risk: ${risk} | Aletheia daemon offline — using local classification`,
    },
  };
}

/**
 * Local fallback for post-tool-use: detect test/verification commands.
 * Returns a HookOutput if a test keyword is found, or null to exit silently.
 */
export function fallbackPostToolUse(input: HookInput): HookOutput | null {
  const cmd = ((input.tool_input as Record<string, unknown>)?.command as string || "").toLowerCase();
  const testKw = ["test", "pytest", "cargo test", "cargo check", "lint", "tsc", "mypy"];
  if (testKw.some(k => cmd.includes(k))) {
    return {
      hookSpecificOutput: {
        hookEventName: "PostToolUse",
        additionalContext: "Verification signal captured. Treat results as evidence.",
      },
    };
  }
  return null;
}

/**
 * Local fallback for stop: remind to produce a proof packet.
 * Always returns a HookOutput (never null).
 */
export function fallbackStop(): HookOutput {
  return {
    hookSpecificOutput: {
      hookEventName: "Stop",
      additionalContext: "Finish with proof packet: Goal, Changes, Evidence, Uncertainty, Next step.",
    },
  };
}
