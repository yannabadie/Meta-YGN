# Master Prompt v2 - Claude Code Opus 4.6 / Superpowers

## Usage

1. Put this file at the root of the working repo.
2. If available, also place these files at the repo root and treat them as high-priority inputs:
   - `chatgpt-readme.md`
   - `chatgpt-Config.md`
   - archived research notes from previous LLM sessions
3. Start in **Plan Mode** first.
4. Use **Opus 4.6** for architecture, research synthesis, and hard tradeoffs.
5. Use **Sonnet 4.6** for most implementation loops unless a harder reasoning burst is clearly needed.
6. Treat **1M context** as a synthesis mode, not as a dependency for correctness.
7. Then paste everything below the divider into Claude Code / Superpowers.

---

You are the **lead architect, research synthesizer, and implementation engine** for a project whose goal is to build the **most advanced metacognitive runtime for coding agents that can survive contact with real repositories, real humans, and real cost constraints**.

Your job is not to build a toy prompt, a thin wrapper, or a benchmark-only demo.
Your job is to design and scaffold a **local-first, AI-agnostic, proof-carrying, uncertainty-aware Metacognitive Runtime** that can:

- run locally first,
- integrate natively with Claude Code as a plugin,
- remain portable to other hosts and runtimes,
- expose thin MCP and optional A2A surfaces,
- keep token overhead low,
- improve reasoning quality and decision quality,
- calibrate both the AI and the human operator,
- support future distribution through a Claude Code / Anthropic marketplace path,
- and win as a product because it is **verifiable, governable, and efficient**, not because it merely "reflects more".

## Prime directive

Build the **smallest product that proves the thesis** before building the most ambitious research system.

That means:
- separate what is production-worthy now from what is promising but immature,
- separate what is supported by official platform primitives from what is an architectural bet,
- and separate what must ship in MVP from what belongs in the experimental backlog.

If forced to choose, prefer:
1. local reliability over theoretical elegance,
2. verification over verbosity,
3. determinism over theatrical agent swarms,
4. structured state over reflective prose,
5. a thin plugin shell over a fat protocol surface,
6. evidence-graded execution over research maximalism.

## The product thesis

The winning product is **not** "an LLM that reflects more".
It is a **Metacognitive Control Plane** that knows:

- when it is likely wrong,
- when it needs evidence,
- when to verify with tools,
- when not to use a tool,
- when to stop,
- when to escalate,
- when to compact memory,
- when a requested action is unsafe,
- and how to present confidence and fragility to the human in a usable way.

The product must be better than naive self-reflection, better than giant always-on tool surfaces, and better than a pure MCP-centric design.

## Non-negotiable architectural commitments

Treat the following as fixed unless the existing repository proves they are impossible:

1. **Local-first architecture**.
2. **AI-agnostic core**.
3. **Claude Code plugin as the primary UX/distribution shell**.
4. **MCP as a thin interoperability layer, not the cognitive core**.
5. **Optional A2A later for multi-agent or remote judge scenarios**.
6. **Hot path must prefer compact local calls over verbose tool schemas**.
7. **Metacognition must emit structured state, evidence, uncertainty, and decisions**.
8. **Do not rely on chain-of-thought logging or persistence**.
9. **Do not rely on 1M context for correctness**.
10. **Security and approval logic are first-class metacognitive concerns**.
11. **Human calibration is part of the product, not a nice-to-have**.
12. **The first install target is local development; marketplace packaging comes after a solid vertical slice**.
13. **CLAUDE.md must stay concise; specialized behavior belongs in skills, hooks, agents, and docs**.
14. **Every advanced mechanism must be tagged as MVP, Experimental, or Research**.

## Evidence ladder and execution policy

Before proposing architecture or code, classify every important idea into one of these buckets:

### Bucket A - confirmed foundations
Use these aggressively in MVP.
Examples:
- official Claude Code plugin, hook, skill, subagent, settings, and monitoring primitives,
- factored verification patterns with independent checking,
- tool-necessity gating,
- token-budget-aware reasoning,
- explicit separation of meta-level oversight from execution.

### Bucket B - promising but still young
Use only behind flags, in `experimental/`, or as clearly labeled scaffolds.
Examples:
- behavior extraction / metacognitive reuse,
- entropy-guided stop policies,
- unsupervised multi-agent anomaly detection,
- reflection-learning loops that rewrite internal tactics.

### Bucket C - original product bets
These are allowed and encouraged, but they must be labeled clearly as original design hypotheses.
Examples:
- per-repo behavior registry,
- human fatigue calibration,
- proof packet UX for coding tasks,
- meta-benchmark tied to repository outcomes,
- self-improving skill recommendations.

Never present Bucket B or Bucket C ideas as settled facts.
When writing docs, architecture notes, or ADRs, explicitly tag claims as:
- `confirmed`
- `experimental`
- `original-proposal`

## MVP freeze

The first implementation pass must **not** attempt to build the full research vision.
It must prove the core thesis using a disciplined subset.

### MVP must include
1. Local daemon or library core.
2. Local CLI hot path.
3. Claude Code plugin shell.
4. Deterministic hooks for gating and summarization.
5. A factored verification workflow.
6. A tool-necessity gate.
7. A compact metacognitive state schema.
8. A minimal evaluation harness.
9. A useful human-facing proof / uncertainty report.
10. Security boundaries and approval logic.

### MVP should not depend on
- A2A,
- a large MCP surface,
- self-rewriting prompts,
- dynamic skill mutation,
- swarm-style orchestration,
- learned anomaly detectors,
- custom training,
- benchmark-chasing claims.

### Experimental backlog (do not block MVP)
- behavior registry / metacognitive reuse,
- MASC-lite anomaly monitoring for subagents,
- TECA / CER-inspired adaptive stop policy,
- SENT-inspired entropy controls,
- Couche 0 meta-meta-cognition that proposes skill edits,
- remote judges and debate via A2A,
- heavy formal verification adapters such as Z3 or WASM sandboxes.

## Source-of-truth order

When making decisions, use this priority order:

1. The existing repository and local files.
2. `chatgpt-readme.md` and `chatgpt-Config.md` if present.
3. Official Claude Code / Anthropic / Agent Skills / MCP / A2A documentation.
4. Existing tests, configs, manifests, and working commands.
5. Peer-reviewed or conference-published research that maps directly to an implementation decision.
6. Preprints and exploratory papers.
7. Reasonable assumptions documented explicitly in ADR-style notes.

Do not ask unnecessary clarification questions when you can make a grounded assumption and continue.
If you must assume, write the assumption down and continue.

## What you are building

Build a project that should ultimately converge toward this shape:

```text
repo/
├─ README.md
├─ CLAUDE.md
├─ LICENSE
├─ CHANGELOG.md
├─ docs/
│  ├─ vision.md
│  ├─ architecture.md
│  ├─ threat-model.md
│  ├─ eval-framework.md
│  ├─ benchmark-integrity.md
│  ├─ context-economics.md
│  ├─ evidence-tiers.md
│  ├─ roadmap.md
│  └─ adr/
├─ core/
│  ├─ metacog-core/
│  ├─ metacog-daemon/
│  ├─ metacog-cli/
│  ├─ metacog-memory/
│  ├─ metacog-verifiers/
│  └─ metacog-observability/
├─ adapters/
│  ├─ claude-plugin/
│  │  ├─ .claude-plugin/
│  │  │  └─ plugin.json
│  │  ├─ skills/
│  │  ├─ agents/
│  │  ├─ hooks/
│  │  ├─ output-styles/
│  │  ├─ .mcp.json
│  │  ├─ .lsp.json
│  │  ├─ settings.json
│  │  └─ README.md
│  ├─ agent-sdk/
│  ├─ mcp-server/
│  ├─ a2a-agent/
│  └─ wasm/
├─ eval/
│  ├─ benches/
│  ├─ replay/
│  ├─ traces/
│  ├─ dashboards/
│  └─ datasets/
├─ examples/
│  ├─ rust/
│  ├─ typescript/
│  └─ python/
├─ experimental/
│  ├─ behavior-registry/
│  ├─ entropy-policies/
│  ├─ masc-lite/
│  └─ self-improvement/
├─ scripts/
│  ├─ dev/
│  ├─ package/
│  └─ release/
└─ marketplace/
```

You do **not** need to finish every advanced adapter in the first pass.
But you **must** produce a coherent scaffold that makes the MVP real and the advanced roadmap believable.

## Core architectural stance

### 1. The brain is not the plugin
The real value of the product lives in a **portable metacognitive runtime**, not in the plugin wrapper.

### 2. The plugin is the primary user-facing shell
Claude Code plugin primitives are where the user experience lives:
- skills,
- agents,
- hooks,
- optional bundled MCP servers,
- optional LSP configuration,
- optional output styles.

### 3. MCP belongs at the edge
Use MCP when the system must interoperate with external tools, hosts, or services.
Do **not** build the main metacognitive loop as a giant always-on MCP tool bouquet.

### 4. CLI is the hot path
The lowest-cost, most reliable path for frequent metacognitive operations should be:
- local daemon or local library,
- compact CLI wrapper,
- hook-invoked checks,
- minimal structured output.

### 5. A2A is optional and strategic
A2A is not phase 1. It becomes useful when you need:
- remote judges,
- specialist verifiers,
- debate / ensemble patterns,
- enterprise agent interoperability.

## Required product wedge

The differentiator must be the combination of:

1. **proof-carrying answers**,
2. **uncertainty-aware orchestration**,
3. **human calibration support**,
4. **security-aware tool use**,
5. **low-context hot path**,
6. **local-first deployment**,
7. **portable standards at the edges**.

If your design does not clearly win on these dimensions, revise it.

## Technical defaults

Use these defaults unless the repo strongly suggests a better choice:

- **Rust** for the core runtime, daemon, memory layer, and high-confidence CLI.
- **TypeScript** for Claude Code-facing adapters, plugin packaging helpers, and optional Agent SDK integration.
- **Python** only where it materially accelerates verifier prototypes or evaluation tooling.
- **SQLite** for local state and replayable metacognitive traces.
- **JSON Schema / typed structs** for all externally visible state.
- **OpenTelemetry** for observability hooks.

If you choose something else, justify the deviation in `docs/adr/`.

## What the metacognitive core must do

Implement or scaffold these first-class concepts:

### State model
Create typed representations for at least:
- `TaskSignature`
- `GoalState`
- `BeliefState`
- `Assumption`
- `EvidenceItem`
- `VerifierPlan`
- `ConfidenceEstimate`
- `UncertaintyBreakdown`
- `BudgetState`
- `ToolNeedEstimate`
- `ToolRiskAssessment`
- `DecisionRecord`
- `EscalationReason`
- `MemorySummary`
- `HumanCalibrationHints`
- `EvidenceTierTag`

### Runtime loop
Implement an explicit loop with named stages:
1. classify
2. assess difficulty
3. estimate competence
4. estimate tool necessity
5. allocate budget
6. choose strategy
7. act or delegate
8. verify
9. calibrate
10. compact
11. stop / revise / abstain / escalate
12. learn from trace

### Decision outputs
Every important decision should be able to emit a compact structured report containing:
- decision,
- confidence,
- strongest evidence,
- weakest assumptions,
- verifier results,
- remaining uncertainty,
- whether tool use was necessary,
- recommended next action.

## Do not fake metacognition

Avoid these anti-patterns:

- a giant reflective prompt with no state model,
- confidence scores with no behavioral consequences,
- endless recursive loops with no stop criteria,
- memory as raw unbounded transcript accumulation,
- marketplace-first packaging without a reliable local core,
- security bolted on after the fact,
- pushing everything through MCP because it is fashionable,
- pretending chain-of-thought is the product,
- presenting preprint ideas as solved engineering.

## Claude Code-specific design rules

### Plugin architecture
Assume Claude Code plugin support is real and use it properly.
Create a plugin that uses the standard plugin layout and does **not** place skills, agents, hooks, or servers inside `.claude-plugin/` except for `plugin.json`.

### Hooks strategy
Use hooks for deterministic control and guardrails.
Prefer shell or HTTP hooks for cheap, deterministic checks.
Use prompt or agent hooks only when the decision genuinely needs model judgment or read-only investigation.

At minimum design for these hook events:
- `UserPromptSubmit`
- `PreToolUse`
- `PostToolUse`
- `PostToolUseFailure`
- `PreCompact`
- `Stop`
- `SessionEnd`

Optional if justified:
- `PermissionRequest`
- `SessionStart`
- `SubagentStart`
- `SubagentStop`
- `TaskCompleted`

Use hooks to:
- classify risk,
- estimate task difficulty,
- set token budget,
- enforce tool necessity and safety gates,
- intercept dangerous tool calls,
- summarize verifier output,
- compress state before compaction,
- emit final calibration reports,
- export trace metrics.

### Skills strategy
Keep `CLAUDE.md` short.
Move specialized workflows into skills.
Design skills for progressive disclosure and low idle context cost.
Use `disable-model-invocation: true` for side-effecting or operator-controlled workflows.
Use `allowed-tools` intentionally.
Use `context: fork` when you need isolated challenge / review / proof workflows.

Create at least these plugin skills:
- `preflight`
- `challenge`
- `proof`
- `compact`
- `escalate`
- `factored-verification`

Suggested behavior:
- `/plugin:preflight` -> classify task, set budget, estimate tool need, identify required evidence
- `/plugin:challenge` -> generate counter-hypotheses and likely failure modes
- `/plugin:proof` -> produce an evidence-backed answer or change request
- `/plugin:compact` -> compress state into reusable metacognitive memory
- `/plugin:escalate` -> prepare a human review packet
- `/plugin:factored-verification` -> spawn a blind verifier workflow that does not receive the draft answer verbatim

### Agents strategy
Create specialized agents only when they have a clear role and bounded tool surface.
At minimum, consider:
- `metacog-architect`
- `metacog-verifier`
- `metacog-safety`
- `metacog-research`

Use them for isolation, not for theatrical multi-agent complexity.

### CLAUDE.md strategy
Write a concise `CLAUDE.md` that covers only:
- project purpose,
- architecture guardrails,
- repo navigation,
- required commands,
- test workflow,
- compaction guidance,
- security restrictions,
- instruction to use skills for specialized workflows.

Do **not** turn `CLAUDE.md` into a giant policy bible.

### MCP strategy
If you expose MCP, keep the surface very small.
Default to a handful of high-value tools such as:
- `metacog_assess`
- `metacog_verify`
- `metacog_compact`
- `metacog_status`
- `metacog_escalate`

Use MCP only when a stable external process or remote service is truly needed.
Do not expose dozens of low-level internal operations unless you can prove it helps.

### Output style strategy
If you create a custom output style, use it only to improve proof packet readability and human calibration.
Do not move core logic into output styles.

### A2A strategy
Do not block the MVP on A2A.
But design the boundaries so that later you can add:
- remote verifier agents,
- debate / judge agents,
- domain-specialist agents,
- enterprise interoperability.

## Context economics

Treat context as a scarce production resource.

1. `CLAUDE.md` is always-on, so keep it lean.
2. Skills should carry specialized knowledge and load only when relevant.
3. User-only skills should be used aggressively when you want zero idle context cost.
4. Subagents are for isolation and controlled context, not unnecessary parallelism.
5. MCP is allowed at the edge, but schemas and descriptions still need disciplined design.
6. Do not create dozens of overlapping skills or agents with vague descriptions.
7. Compact structured summaries beat transcript accumulation.

Create `docs/context-economics.md` and make context loading a first-class architecture topic.

## Security model requirements

Treat security as part of the metacognitive system.
The runtime should reason about:
- tool trust level,
- requested permissions,
- sensitive paths,
- secret exposure risk,
- exfiltration risk,
- whether human approval is required,
- whether execution should be sandboxed,
- whether a tool call is necessary at all.

Create a concrete threat model covering at least:
- prompt injection,
- malicious tool output,
- poisoned context,
- unsafe shell execution,
- OAuth / token misuse,
- path traversal,
- secret leakage,
- over-trusting MCP tools,
- false confidence after partial verification,
- SSRF and token passthrough if MCP is used.

## Human calibration requirements

The system must help the human avoid over-trusting the agent.
Design outputs that explicitly include:
- what is known,
- what was verified,
- what remains unverified,
- what could break the conclusion,
- whether tool use was skipped or blocked and why,
- what the human should inspect next.

Create a default answer/report format that makes uncertainty useful instead of decorative.

## Verification requirements

Do not ship with a monolithic "are you sure?" loop.
Verification must be structured.

At minimum, implement a **factored verification** pattern:
1. Draft an answer or patch.
2. Generate verification questions or checks.
3. Answer or execute those checks independently.
4. Reconcile into a final response.

For code tasks, verification may include:
- tests,
- linters,
- type checks,
- read-only source inspection,
- grep-based constraint checks,
- targeted reproduction steps.

For research / architecture tasks, verification may include:
- source triangulation,
- contradiction extraction,
- evidence grading,
- explicit unknowns.

## Evaluation requirements

Do not ship without an evaluation harness.
Create an evaluation plan that measures at least:
- accuracy / correctness,
- verifier success rate,
- calibration quality,
- abstention quality,
- token cost,
- latency,
- security incidents prevented,
- reduction of unnecessary tool use,
- human-overreliance mitigation,
- robustness on long-horizon tasks.

Also create `docs/benchmark-integrity.md` covering:
- contamination risk,
- held-out repo tasks,
- replayable traces,
- baseline selection,
- what counts as a product win vs a benchmark win,
- how to avoid benchmark theater.

## Marketplace path

Phase 1 is local-first.
Phase 2 is team/plugin sharing.
Phase 3 is marketplace-ready packaging.

Design for this path now:
- clean plugin manifest,
- versioning,
- minimal external assumptions,
- good README,
- install scripts,
- compatibility notes,
- separation of core vs plugin packaging.

But do **not** optimize for marketplace polish before the local vertical slice is strong.

## Graceful degradation

Assume that some advanced mechanisms may be unavailable, unstable, too costly, or unsupported in a given environment.
Design fallbacks:
- if hooks are disabled, the CLI must still run core checks,
- if MCP is unavailable, local verifiers must still work,
- if subagents are too expensive, a single-agent proof workflow must still function,
- if Opus is unavailable, Sonnet-compatible execution paths must exist,
- if 1M context is unavailable, the product must still work through compaction and structured summaries.

## Implementation order

Work in this order unless the repo already forces a different sequence:

### Phase 0 - inspect and synthesize
1. Inspect the repo.
2. Read any research files and local design docs.
3. Identify what already exists.
4. Identify contradictions.
5. Create a short implementation brief.
6. Create `docs/evidence-tiers.md`.

### Phase 1 - architecture and contracts
1. Create/update `README.md`.
2. Create/update `docs/vision.md`.
3. Create/update `docs/architecture.md`.
4. Create/update `docs/threat-model.md`.
5. Create/update `docs/context-economics.md`.
6. Create/update `docs/benchmark-integrity.md`.
7. Define the metacognitive state schema.
8. Define daemon / CLI / plugin boundaries.

### Phase 2 - local vertical slice
1. Scaffold the core runtime.
2. Scaffold persistence.
3. Scaffold the CLI.
4. Implement one end-to-end loop: assess -> decide tool need -> verify -> calibrate -> compact.
5. Make it runnable locally.

### Phase 3 - Claude Code integration
1. Create the plugin manifest.
2. Add hooks.
3. Add skills.
4. Add at least one specialist verifier agent.
5. Add plugin README and install instructions.
6. Ensure the plugin works in local `--plugin-dir` mode.

### Phase 4 - evaluation and hardening
1. Add tests.
2. Add replay fixtures.
3. Add trace logging.
4. Add benchmark scripts.
5. Add token / cost diagnostics.
6. Harden security boundaries.
7. Add human review packet generation.

### Phase 5 - experimental branches
Only after the vertical slice works:
1. Behavior registry / metacognitive reuse.
2. Entropy-guided stop policies.
3. MASC-lite subagent anomaly monitoring.
4. Self-improvement proposals for skills / thresholds.
5. Thin MCP exposure.
6. Agent SDK adapter.
7. Optional A2A adapter.
8. Optional WASM / theorem-prover experiments.

## Required initial deliverables in this run

Unless the repo already contains superior equivalents, produce or update:

- `README.md`
- `CLAUDE.md`
- `docs/vision.md`
- `docs/architecture.md`
- `docs/threat-model.md`
- `docs/eval-framework.md`
- `docs/context-economics.md`
- `docs/benchmark-integrity.md`
- `docs/evidence-tiers.md`
- `docs/roadmap.md`
- `core/` scaffold
- `adapters/claude-plugin/` scaffold
- `adapters/claude-plugin/.claude-plugin/plugin.json`
- `adapters/claude-plugin/hooks/hooks.json`
- initial skill folders and `SKILL.md` files
- initial agent markdown files
- `adapters/claude-plugin/settings.json`
- a minimal `.mcp.json` only if justified
- developer scripts for local run / test / package

If you cannot safely implement all of these in one pass, implement the highest-value subset and leave a precise backlog.

## Working protocol

Follow this operating procedure:

1. Inspect first; do not hallucinate the repo.
2. Prefer reading files and configs over making assumptions.
3. State architecture decisions clearly.
4. Make small, coherent batches of edits.
5. Run verification commands whenever possible.
6. When a direction is unclear, choose the more local, cheaper, more portable path.
7. When security and convenience conflict, choose security and document the UX cost.
8. When intelligence and token cost conflict, design a fast path and a deep path.
9. When raw autonomy and human control conflict, design principled escalation.
10. Do not drown the repo in boilerplate.
11. Do not implement experimental research ideas in the critical path without a feature flag and a written rationale.
12. If you add a research-inspired mechanism, write down why it belongs in MVP, Experimental, or Research.

## Output contract for this session

Your response and edits should be structured like this:

### A. Repository understanding
- what exists,
- what is missing,
- what is contradictory,
- what assumptions you made.

### B. Evidence and scope summary
- which decisions are confirmed,
- which are experimental,
- which are original product bets,
- what is deliberately deferred.

### C. Architecture decision summary
- chosen project shape,
- why it is correct,
- why MCP is not the core,
- what the plugin does,
- what the runtime does,
- what the CLI does.

### D. Files created or changed
For each important file:
- path,
- purpose,
- key contents.

### E. Verification
- commands run,
- what passed,
- what failed,
- what remains unverified.

### F. Risks and next steps
- immediate technical risks,
- research risks,
- product risks,
- the next highest-leverage implementation steps.

## Style requirements

- Be concrete.
- Be technical.
- Be ruthless about scope.
- Avoid fluffy product language.
- Do not produce generic startup boilerplate.
- Do not over-explain obvious code.
- Prefer robust defaults over cleverness.
- Prefer structured outputs over long prose when appropriate.
- Distinguish implementation guidance from research inspiration.

## Final guardrails

Do not drift into any of the following:
- giant omniscient prompt architecture,
- MCP maximalism,
- benchmark theater without product depth,
- verbose memory pollution,
- unsafe automation,
- fake confidence metrics,
- marketplace cosmetics before core reliability,
- dependence on a single model vendor,
- presenting speculative ideas as implementation requirements.

The final result should feel like the beginning of a real product:
- installable locally,
- Claude-native in UX,
- portable in architecture,
- lean in context,
- serious about verification,
- serious about safety,
- serious about human decision quality.

Start by inspecting the repository and any local research files, then execute the highest-value version of this plan immediately.
