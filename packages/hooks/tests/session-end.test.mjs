import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const sessionEndBundle = join(packageRoot, "dist", "session-end.mjs");

test("session-end notifies the daemon with bearer auth when token file exists", async (t) => {
  const homeDir = await mkdtemp(join(tmpdir(), "metaygn-hooks-"));
  const daemonDir = join(homeDir, ".claude", "aletheia");
  await mkdir(daemonDir, { recursive: true });

  let capturedRequest = null;
  const server = createServer((req, res) => {
    let body = "";
    req.setEncoding("utf8");
    req.on("data", (chunk) => {
      body += chunk;
    });
    req.on("end", () => {
      capturedRequest = {
        authorization: req.headers.authorization ?? null,
        body,
      };
      res.statusCode = 200;
      res.setHeader("Content-Type", "application/json");
      res.end("{}");
    });
  });

  t.after(async () => {
    await new Promise((resolve, reject) => {
      server.close((error) => {
        if (error) {
          reject(error);
        } else {
          resolve();
        }
      });
    });
    await rm(homeDir, { recursive: true, force: true });
  });

  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert.ok(address && typeof address === "object");

  await writeFile(join(daemonDir, "daemon.port"), String(address.port));
  await writeFile(join(daemonDir, "daemon.token"), "secret-token\n");

  const input = {
    hook_event_name: "SessionEnd",
    session_id: "sess-auth-test",
    cwd: "C:/Code/Meta-YGN",
  };

  const child = spawn(process.execPath, [sessionEndBundle], {
    cwd: packageRoot,
    env: {
      ...process.env,
      HOME: homeDir,
      USERPROFILE: homeDir,
    },
    stdio: ["pipe", "ignore", "pipe"],
  });

  child.stdin.end(JSON.stringify(input));

  const [exitCode] = await once(child, "close");
  assert.equal(exitCode, 0);

  const deadline = Date.now() + 750;
  while (capturedRequest === null && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 25));
  }

  assert.ok(capturedRequest, "expected session-end to notify the daemon");
  assert.equal(capturedRequest.authorization, "Bearer secret-token");
  assert.deepEqual(JSON.parse(capturedRequest.body), input);
});
