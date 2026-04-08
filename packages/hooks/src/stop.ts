/**
 * stop hook entry point.
 *
 * Reads JSON from stdin, tries the daemon, falls back to proof packet reminder.
 * Run via: bun run packages/hooks/src/stop.ts
 *   or:   node --experimental-strip-types packages/hooks/src/stop.ts
 *   or:   npx tsx packages/hooks/src/stop.ts
 */

import { HookInputSchema } from "@metaygn/shared/src/types.js";
import type { HookInput, HookOutput } from "@metaygn/shared/src/types.js";
import { callDaemon } from "./lib/daemon-client.js";
import { fallbackStop } from "./lib/fallback.js";
import { readStdin, respond } from "./lib/stdin.js";

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
  const daemonResult = await callDaemon("/hooks/stop", input);
  if (daemonResult?.hookSpecificOutput) {
    respond(daemonResult);
    return;
  }

  // 2. Fall back to proof packet reminder
  const fallbackResult = fallbackStop();
  respond(fallbackResult);
}

main().catch(() => {
  // Unhandled error — exit silently (allow)
  process.exit(0);
});
