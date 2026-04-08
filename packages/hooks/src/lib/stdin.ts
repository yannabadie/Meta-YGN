/**
 * Shared stdin/stdout helpers for hook entry points.
 *
 * Extracted to avoid duplicating readStdin() and respond() in every hook file.
 */

// Augment globalThis for Bun runtime detection
declare const Bun: { stdin: { json(): Promise<unknown> } } | undefined;

/**
 * Read JSON from stdin. Supports both Bun and Node.js runtimes.
 */
export async function readStdin(): Promise<unknown> {
  if (typeof Bun !== "undefined") {
    return Bun.stdin.json();
  }
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    process.stdin.on("data", (chunk: Buffer) => chunks.push(chunk));
    process.stdin.on("end", () => {
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString("utf-8")));
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
export function respond(output: unknown): void {
  process.stdout.write(JSON.stringify(output) + "\n");
  process.exit(0);
}
