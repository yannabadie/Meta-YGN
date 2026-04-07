/**
 * Bundle all hook entry points into self-contained .mjs files.
 *
 * Produces packages/hooks/dist/<hook-name>.mjs for each hook.
 * Each bundle inlines all dependencies (zod, @metaygn/shared, lib/*)
 * so the hooks can run with plain `node` -- no npx tsx overhead.
 *
 * Usage: node scripts/bundle.mjs
 */

import { build } from "esbuild";
import { readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const srcDir = join(__dirname, "..", "src");
const outDir = join(__dirname, "..", "dist");

// Discover all top-level .ts entry points (exclude lib/)
const entryPoints = readdirSync(srcDir)
  .filter((f) => f.endsWith(".ts") && !f.startsWith("lib"))
  .map((f) => join(srcDir, f));

console.log(`Bundling ${entryPoints.length} hooks...`);

await build({
  entryPoints,
  bundle: true,
  platform: "node",
  target: "node20",
  format: "esm",
  outdir: outDir,
  outExtension: { ".js": ".mjs" },
  logLevel: "info",
});

console.log(`Done. Output in ${outDir}`);
