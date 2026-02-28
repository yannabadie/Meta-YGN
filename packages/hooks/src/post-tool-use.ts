/**
 * post-tool-use hook entry point.
 *
 * Reads JSON from stdin, tries the daemon, falls back to local test-detection heuristic.
 * Run via: bun run packages/hooks/src/post-tool-use.ts
 *   or:   node --experimental-strip-types packages/hooks/src/post-tool-use.ts
 *   or:   npx tsx packages/hooks/src/post-tool-use.ts
 */

import { HookInputSchema } from "@metaygn/shared/src/types.js";
import type { HookInput, HookOutput } from "@metaygn/shared/src/types.js";
import { callDaemon } from "./lib/daemon-client.js";
import { fallbackPostToolUse } from "./lib/fallback.js";

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
    // If we can't read stdin, exit silently (allow)
    process.exit(0);
  }

  // Validate input against schema
  const parsed = HookInputSchema.safeParse(raw);
  if (!parsed.success) {
    // Invalid input — exit silently (allow)
    process.exit(0);
  }

  const input: HookInput = parsed.data;

  // 1. Try the daemon
  const daemonResult = await callDaemon("/hooks/post-tool-use", input);
  if (daemonResult?.hookSpecificOutput) {
    respond(daemonResult);
    return;
  }

  // 2. Fall back to local test-detection heuristic
  const fallbackResult = fallbackPostToolUse(input);
  if (fallbackResult) {
    respond(fallbackResult);
    return;
  }

  // 3. No opinion — exit silently (allow)
  process.exit(0);
}

main().catch(() => {
  // Unhandled error — exit silently (allow)
  process.exit(0);
});
