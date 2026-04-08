# FAQ

## Performance

**Q: How much latency does Aletheia-Nexus add to each tool call?**

The daemon processes most hook calls in under 5ms. The AST guard (tree-sitter parsing) adds 1-2ms. The TypeScript hooks start in ~50ms thanks to esbuild pre-compilation. Total overhead per tool call is typically under 10ms.

**Q: Does it slow down my Claude Code session?**

Negligibly. The daemon runs as a separate process and returns verdicts synchronously. The 5-layer cascade short-circuits: if Layer 1 (AST Guard) blocks a command with score 0, layers 2-5 are never evaluated.

**Q: How much memory does the daemon use?**

Typical memory usage is 20-40 MB. The SQLite database grows with session history but uses FTS5 for efficient full-text search. Graph memory uses in-process storage with optional embeddings.

---

## Fallback Behavior

**Q: What happens if the daemon is not running?**

TypeScript hooks provide regex-based fallback guards. Coverage is narrower -- they catch common destructive patterns (`rm -rf`, `git push --force`, `terraform destroy`) but cannot do AST parsing or semantic routing. The daemon is always preferred.

**Q: What if a hook fails or times out?**

The system fails open by default -- if a hook cannot reach the daemon and the TypeScript fallback does not match a pattern, the tool call proceeds. This prevents the safety layer from blocking legitimate work during outages.

---

## Comparison with Other Tools

**Q: How does this compare to Guardrails AI / NeMo Guardrails?**

Guardrails AI and NeMo Guardrails focus on LLM input/output filtering (prompt injection, content moderation, topic rails). Aletheia-Nexus focuses on tool call safety -- it understands what shell commands, file operations, and git operations actually do via AST parsing. The two are complementary, not competing.

**Q: How does this compare to LlamaFirewall?**

LlamaFirewall provides prompt injection detection and jailbreak prevention. Aletheia-Nexus includes prompt injection detection as one of its assessment stages, but its primary focus is preventing destructive tool execution (the kind that deletes databases and force-pushes to production).

**Q: Does this replace Claude Code's built-in permission system?**

No. Claude Code's built-in permission prompts still apply. Aletheia-Nexus runs as an additional layer through hooks. It can deny a tool call before Claude Code's own permission check, or add context (risk score, recovery instructions) to the permission prompt.

---

## False Positives

**Q: What if Aletheia-Nexus blocks a safe command?**

The adaptive guard system tracks true positives and false positives per rule. Rules that generate too many false positives are automatically disabled. You can also override individual decisions when the verdict is "ask" (the prompt shows the risk assessment and lets you approve).

**Q: How does it handle `rm` on safe targets?**

The semantic router provides context-aware scoring. `rm target/*.o` (cleaning build artifacts) scores 20 (safe). `rm -rf /` scores 0 (blocked). `rm -rf node_modules` scores intermediate. The AST guard checks whether the command targets root, uses recursive+force flags, or deletes without a specific path.

**Q: Can I tune the sensitivity?**

The heuristic evolution system adjusts risk weights based on session outcomes. Over time, the system calibrates to your usage patterns. You can also view and evolve heuristics via the `/heuristics/*` API endpoints.

---

## Contributing

**Q: How do I run the test suite?**

```bash
cargo test --workspace
```

There are 774+ tests across all crates. To run tests for a specific crate:

```bash
cargo test -p metaygn-core
cargo test -p metaygn-daemon
```

**Q: How do I add a new guard rule?**

Guard rules live in the core crate. The AST guard uses tree-sitter grammars. Pattern-based rules are defined in the guard pipeline. Add new patterns, write tests, and verify with `cargo test -p metaygn-core`.

**Q: How do I report a false positive?**

Open a GitHub issue with the command that was incorrectly blocked, the risk score, and the daemon version. Include the hook response JSON if possible.

**Q: What is the license?**

MIT. See [LICENSE](https://github.com/yannabadie/Meta-YGN/blob/master/LICENSE).
