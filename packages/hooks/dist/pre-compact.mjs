// src/pre-compact.ts
function respond(output) {
  process.stdout.write(JSON.stringify(output) + "\n");
  process.exit(0);
}
function main() {
  respond({
    hookSpecificOutput: {
      hookEventName: "PreCompact",
      additionalContext: "Compact into: 1) Current goal, 2) Verified facts, 3) Failed approaches, 4) Open risks, 5) Next action"
    }
  });
}
main();
