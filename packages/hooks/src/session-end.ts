/**
 * session-end hook entry point.
 *
 * Fire-and-forget: sends session data to daemon and exits immediately.
 * Does NOT wait for a response. On any error, exits silently.
 * Run via: bun run packages/hooks/src/session-end.ts
 *   or:   node --experimental-strip-types packages/hooks/src/session-end.ts
 *   or:   npx tsx packages/hooks/src/session-end.ts
 */

import { HookInputSchema } from "@metaygn/shared/src/types.js";
import type { HookInput } from "@metaygn/shared/src/types.js";
import { readStdin } from "./lib/stdin.js";
import { notifyDaemon } from "./lib/daemon-client.js";

async function main(): Promise<void> {
  let raw: unknown;
  try {
    raw = await readStdin();
  } catch {
    process.exit(0);
  }

  const parsed = HookInputSchema.safeParse(raw);
  if (!parsed.success) {
    process.exit(0);
  }

  const input: HookInput = parsed.data;

  // Best-effort notify the daemon. Bounded timeout keeps the hook non-blocking
  // while still guaranteeing the request is actually attempted.
  await notifyDaemon("/hooks/session-end", input);

  // Exit immediately regardless of daemon response
  process.exit(0);
}

main().catch(() => {
  process.exit(0);
});
