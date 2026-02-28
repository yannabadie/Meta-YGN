/**
 * post-tool-use-failure hook entry point.
 *
 * Reads JSON from stdin, tries the daemon, falls back to generic error diagnosis hint.
 * Always emits something (never silent on failure).
 * Run via: bun run packages/hooks/src/post-tool-use-failure.ts
 *   or:   node --experimental-strip-types packages/hooks/src/post-tool-use-failure.ts
 *   or:   npx tsx packages/hooks/src/post-tool-use-failure.ts
 */

import { HookInputSchema } from "@metaygn/shared/src/types.js";
import type { HookInput, HookOutput } from "@metaygn/shared/src/types.js";
import { callDaemon } from "./lib/daemon-client.js";

// Augment globalThis for Bun runtime detection
declare const Bun: { stdin: { json(): Promise<unknown> } } | undefined;

/**
 * Read JSON from stdin. Supports both Bun and Node.js runtimes.
 */
async function readStdin(): Promise<unknown> {
  // Bun runtime: use Bun.stdin.json()
  if (typeof Bun !== "undefined") {
    return Bun.stdin.json();
  }

  // Node.js runtime: read chunks from process.stdin
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    process.stdin.on("data", (chunk: Buffer) => chunks.push(chunk));
    process.stdin.on("end", () => {
      try {
        const text = Buffer.concat(chunks).toString("utf-8");
        resolve(JSON.parse(text));
      } catch (err) {
        reject(err);
      }
    });
    process.stdin.on("error", reject);
  });
}

/**
 * Write hook output to stdout and exit.
 */
function respond(output: HookOutput): void {
  process.stdout.write(JSON.stringify(output) + "\n");
  process.exit(0);
}

async function main(): Promise<void> {
  let raw: unknown;
  try {
    raw = await readStdin();
  } catch {
    // If we can't read stdin, emit generic fallback (never silent on failure)
    respond({
      hookSpecificOutput: {
        hookEventName: "PostToolUseFailure",
        additionalContext: "Tool failed. Diagnose: check error message, verify inputs, consider alternative approach.",
      },
    });
    return;
  }

  // Validate input against schema
  const parsed = HookInputSchema.safeParse(raw);
  if (!parsed.success) {
    // Invalid input — still emit something on failure
    respond({
      hookSpecificOutput: {
        hookEventName: "PostToolUseFailure",
        additionalContext: "Tool failed. Diagnose: check error message, verify inputs, consider alternative approach.",
      },
    });
    return;
  }

  const input: HookInput = parsed.data;

  // 1. Try the daemon (same endpoint as post-tool-use — daemon handles both)
  const daemonResult = await callDaemon("/hooks/post-tool-use", input);
  if (daemonResult?.hookSpecificOutput) {
    respond(daemonResult);
    return;
  }

  // 2. Fallback — always emit generic error diagnosis hint
  const toolName = input.tool_name ?? "unknown";
  const errorSnippet = input.error ? input.error.slice(0, 120) : "no error details";
  respond({
    hookSpecificOutput: {
      hookEventName: "PostToolUseFailure",
      additionalContext: `Tool "${toolName}" failed (${errorSnippet}). Diagnose: check error message, verify inputs, consider alternative approach.`,
    },
  });
}

main().catch(() => {
  // Unhandled error — exit silently (allow)
  process.exit(0);
});
