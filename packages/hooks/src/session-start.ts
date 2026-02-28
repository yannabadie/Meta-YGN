/**
 * session-start hook entry point.
 *
 * Pings the daemon /health endpoint (GET) to verify it's alive.
 * On success, emits the daemon health response as context.
 * On fallback, emits a static "profile active" message.
 *
 * Run via: bun run packages/hooks/src/session-start.ts
 *   or:   node --experimental-strip-types packages/hooks/src/session-start.ts
 *   or:   npx tsx packages/hooks/src/session-start.ts
 */

import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import type { HookOutput } from "@metaygn/shared/src/types.js";

const DAEMON_PORT_FILE = join(homedir(), ".claude", "aletheia", "daemon.port");
const TIMEOUT_MS = 350;

/**
 * Write hook output to stdout and exit.
 */
function respond(output: HookOutput): void {
  process.stdout.write(JSON.stringify(output) + "\n");
  process.exit(0);
}

/**
 * Read the daemon port from the well-known port file.
 */
async function readDaemonPort(): Promise<number | null> {
  try {
    const raw = await readFile(DAEMON_PORT_FILE, "utf-8");
    const port = parseInt(raw.trim(), 10);
    return Number.isFinite(port) ? port : null;
  } catch {
    return null;
  }
}

async function main(): Promise<void> {
  // 1. Try the daemon health endpoint (GET)
  const port = await readDaemonPort();
  if (port !== null) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
    try {
      const response = await fetch(`http://127.0.0.1:${port}/health`, {
        method: "GET",
        signal: controller.signal,
      });
      if (response.ok) {
        const data = await response.json() as Record<string, unknown>;
        respond({
          hookSpecificOutput: {
            hookEventName: "SessionStart",
            additionalContext: `Aletheia daemon online: ${JSON.stringify(data)}`,
          },
        });
        return;
      }
    } catch {
      // Daemon unreachable — fall through to fallback
    } finally {
      clearTimeout(timer);
    }
  }

  // 2. Fallback — emit static profile-active message
  respond({
    hookSpecificOutput: {
      hookEventName: "SessionStart",
      additionalContext: "Aletheia profile active.",
    },
  });
}

main().catch(() => {
  // Unhandled error — exit silently (allow)
  process.exit(0);
});
