// src/session-start.ts
import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
var DAEMON_PORT_FILE = join(homedir(), ".claude", "aletheia", "daemon.port");
var TIMEOUT_MS = 350;
function respond(output) {
  process.stdout.write(JSON.stringify(output) + "\n");
  process.exit(0);
}
async function readDaemonPort() {
  try {
    const raw = await readFile(DAEMON_PORT_FILE, "utf-8");
    const port = parseInt(raw.trim(), 10);
    return Number.isFinite(port) ? port : null;
  } catch {
    return null;
  }
}
async function main() {
  const port = await readDaemonPort();
  if (port !== null) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
    try {
      const response = await fetch(`http://127.0.0.1:${port}/health`, {
        method: "GET",
        signal: controller.signal
      });
      if (response.ok) {
        const data = await response.json();
        respond({
          hookSpecificOutput: {
            hookEventName: "SessionStart",
            additionalContext: `Aletheia daemon online: ${JSON.stringify(data)}`
          }
        });
        return;
      }
    } catch {
    } finally {
      clearTimeout(timer);
    }
  }
  respond({
    hookSpecificOutput: {
      hookEventName: "SessionStart",
      additionalContext: "Aletheia profile active."
    }
  });
}
main().catch(() => {
  process.exit(0);
});
