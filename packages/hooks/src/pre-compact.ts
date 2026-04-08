/**
 * pre-compact hook entry point.
 *
 * No daemon call needed — compaction guidance is static.
 * Emits structured compaction instructions and exits.
 * Run via: bun run packages/hooks/src/pre-compact.ts
 *   or:   node --experimental-strip-types packages/hooks/src/pre-compact.ts
 *   or:   npx tsx packages/hooks/src/pre-compact.ts
 */

import type { HookOutput } from "@metaygn/shared/src/types.js";
import { respond } from "./lib/stdin.js";

function main(): void {
  respond({
    hookSpecificOutput: {
      hookEventName: "PreCompact",
      additionalContext:
        "Compact into: 1) Current goal, 2) Verified facts, 3) Failed approaches, 4) Open risks, 5) Next action",
    },
  });
}

main();
