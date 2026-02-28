/**
 * session-end hook entry point.
 *
 * Fire-and-forget: sends session data to daemon and exits immediately.
 * Does NOT wait for a response. On any error, exits silently.
 * Run via: bun run packages/hooks/src/session-end.ts
 *   or:   node --experimental-strip-types packages/hooks/src/session-end.ts
 *   or:   npx tsx packages/hooks/src/session-end.ts
 */

import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import { HookInputSchema } from "@metaygn/shared/src/types.js";
import type { HookInput } from "@metaygn/shared/src/types.js";

const DAEMON_PORT_FILE = join(homedir(), ".claude", "aletheia", "daemon.port");

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

  // Fire and forget — send to daemon but do NOT wait for response
  const port = await readDaemonPort();
  if (port !== null) {
    // Intentionally not awaiting — fire and forget
    fetch(`http://127.0.0.1:${port}/hooks/session-end`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    }).catch(() => {
      // Ignore errors silently
    });
  }

  // Exit immediately regardless of daemon response
  process.exit(0);
}

main().catch(() => {
  process.exit(0);
});
