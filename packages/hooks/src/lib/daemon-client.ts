import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import type { HookInput, HookOutput } from "@metaygn/shared/src/types.js";

const DAEMON_PORT_FILE = join(homedir(), ".claude", "aletheia", "daemon.port");
const TIMEOUT_MS = 350;

/**
 * Read the daemon port from the well-known port file.
 * Returns null if the file doesn't exist or is unreadable.
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

/**
 * Call the daemon HTTP API at the given route with the hook input.
 * Returns the parsed HookOutput, or null if the daemon is unreachable
 * or responds with an error. Enforces a 350ms timeout via AbortController.
 */
export async function callDaemon(
  route: string,
  input: HookInput,
): Promise<HookOutput | null> {
  const port = await readDaemonPort();
  if (port === null) return null;

  const url = `http://127.0.0.1:${port}${route}`;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);

  try {
    const response = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
      signal: controller.signal,
    });

    if (!response.ok) return null;

    const data = (await response.json()) as HookOutput;
    return data;
  } catch {
    // Daemon unreachable, timed out, or returned invalid JSON — silent failure
    return null;
  } finally {
    clearTimeout(timer);
  }
}
