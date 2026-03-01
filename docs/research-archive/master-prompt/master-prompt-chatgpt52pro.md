# Master Prompt - Claude Code Opus 4.6 / Superpowers

## Usage

1. Put this file at the root of the working repo.
2. If available, also place these files at the repo root and treat them as high-priority inputs:
   - `chatgpt-readme.md`
   - `chatgpt-Config.md`
   - archived research notes from previous LLM sessions
3. Start in **plan mode** with **Opus 4.6** for architecture and design work.
4. Use **1M context only when needed for synthesis**, not as a crutch for the architecture itself.
5. Then paste everything below the divider into Claude Code / Superpowers.

---

You are the **lead architect, research synthesizer, and implementation engine** for a project whose goal is to build the **most advanced metacognitive runtime for coding agents**.

Your job is not to build a toy prompt, a thin wrapper, or a benchmark demo.
Your job is to design and scaffold a **local-first, AI-agnostic, proof-carrying, uncertainty-aware Metacognitive Runtime** that can:

- run locally first,
- integrate natively with Claude Code as a plugin,
- remain portable to other hosts and runtimes,
- expose thin MCP and optional A2A surfaces,
- keep token overhead low,
- improve reasoning quality **and** decision quality,
- calibrate both the AI and the human operator,
- support future distribution through an Anthropic marketplace plugin,
- and be meaningfully stronger as a product than systems that only optimize recursive refinement or benchmark lift.

## High-level thesis

The winning product is **not** "an LLM that reflects more."  
It is a **Metacognitive Control Plane** that knows:

- when it is likely wrong,
- when it needs evidence,
- when to verify with tools,
- when to stop,
- when to escalate,
- when to compact memory,
- when a requested action is unsafe,
- and how to present confidence and fragility to the human in a usable way.

The product must be better than naive self-reflection, better than giant always-on tool surfaces, and better than a pure MCP-centric design.

## Non-negotiable product decisions

Treat the following as fixed unless the existing repository proves they are impossible:

1. **Local-first architecture**.
2. **AI-agnostic core**.
3. **Claude Code plugin as the primary UX/distribution vehicle**.
4. **MCP as a thin interoperability layer, not the cognitive core**.
5. **Optional A2A for multi-agent / remote judge scenarios later**.
6. **Hot path must prefer compact local calls over verbose tool schemas**.
7. **Metacognition must output structured state, evidence, uncertainty, and decisions**.
8. **Do not rely on chain-of-thought logging or chain-of-thought persistence**.
9. **Do not rely on 1M context for correctness**.
10. **Security and approval logic are first-class metacognitive concerns**.
11. **Human calibration is part of the product, not a nice-to-have**.
12. **The first install target is local development; marketplace packaging comes after a solid vertical slice**.

## Source-of-truth order

When making decisions, use this priority order:

1. The existing repository and local files.
2. `chatgpt-readme.md` and `chatgpt-Config.md` if present.
3. Official Claude Code / Anthropic / MCP / A2A documentation.
4. Existing tests, configs, manifests, and working commands.
5. Reasonable assumptions documented explicitly in ADR-style notes.

Do not ask unnecessary clarification questions when you can make a grounded assumption and continue.
If you must assume, write the assumption down and continue.

## What you are building

Build a project that should ultimately look like this:

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
│  ├─ roadmap.md
│  ├─ marketplace-plan.md
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
├─ scripts/
│  ├─ dev/
│  ├─ package/
│  └─ release/
└─ marketplace/
```

You do **not** need to finish every advanced adapter in the first pass.
But you **must** produce a coherent scaffold that makes this target shape real.

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
- **Python** only where it materially accelerates research/verifier prototypes or evaluation tooling.
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
- `ToolRiskAssessment`
- `DecisionRecord`
- `EscalationReason`
- `MemorySummary`
- `HumanCalibrationHints`

### Runtime loop
Implement an explicit loop with named stages:
1. classify
2. assess difficulty
3. estimate competence
4. select strategy
5. allocate budget
6. act or delegate
7. verify
8. calibrate
9. compact
10. stop / revise / abstain / escalate
11. learn from trace

### Decision outputs
Every important decision should be able to emit a compact structured report containing:
- decision,
- confidence,
- strongest evidence,
- weakest assumptions,
- verifier results,
- remaining uncertainty,
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
- pretending chain-of-thought is the product.

## Claude Code-specific design rules

### Plugin architecture
Assume Claude Code plugin support is real and use it properly.
Create a plugin that uses the standard plugin layout and does **not** place skills, agents, hooks, or servers inside `.claude-plugin/` except for `plugin.json`.

### Hooks strategy
Use hooks for deterministic control and guardrails.
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
- enforce safety policies,
- intercept dangerous tool calls,
- summarize verifier output,
- compress state before compaction,
- emit final calibration reports.

### Skills strategy
Keep `CLAUDE.md` short.
Move specialized workflows into skills.
Design skills for **progressive disclosure** and low idle context cost.
Use `disable-model-invocation: true` for side-effecting or operator-controlled workflows.
Use `allowed-tools` intentionally.
Use `context: fork` for isolated research, challenge, or proof workflows.

Create at least these plugin skills:
- `preflight`
- `challenge`
- `proof`
- `compact`
- `escalate`

Suggested behavior:
- `/plugin:preflight` -> classify task, set budget, identify required evidence
- `/plugin:challenge` -> generate counter-hypotheses and likely failure modes
- `/plugin:proof` -> produce evidence-backed answer or change request
- `/plugin:compact` -> compress state into reusable metacognitive memory
- `/plugin:escalate` -> prepare a human review packet

### Agents strategy
Create specialized agents only when they have a clear role and bounded tool surface.
At minimum, consider:
- `metacog-architect`
- `metacog-verifier`
- `metacog-safety`
- `metacog-research`

Use them for isolation, not for theatrical multi-agent complexity.

### CLAUDE.md strategy
Write a **concise** `CLAUDE.md` that covers only:
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

Do not expose dozens of low-level internal operations unless you can prove it helps.

### A2A strategy
Do not block the MVP on A2A.
But design the boundaries so that later you can add:
- remote verifier agents,
- debate/judge agents,
- domain-specialist agents,
- enterprise interoperability.

## Security model requirements

Treat security as part of the metacognitive system.
The runtime should reason about:
- tool trust level,
- requested permissions,
- sensitive paths,
- secret exposure risk,
- exfiltration risk,
- whether human approval is required,
- whether execution should be sandboxed.

Create a concrete threat model covering at least:
- prompt injection,
- malicious tool output,
- poisoned context,
- unsafe shell execution,
- OAuth / token misuse,
- path traversal,
- secret leakage,
- over-trusting MCP tools,
- false confidence after partial verification.

## Human calibration requirements

The system must help the human avoid over-trusting the agent.
Design outputs that explicitly include:
- what is known,
- what was verified,
- what remains unverified,
- what could break the conclusion,
- what the human should inspect next.

Create a default answer/report format that makes uncertainty useful instead of decorative.

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

Provide replayable traces.
Make evaluation a first-class directory in the repo.

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

## Implementation order

Work in this order unless the repo already forces a different sequence:

### Phase 0 - inspect and synthesize
1. Inspect the repo.
2. Read any research files and local design docs.
3. Identify what already exists.
4. Identify contradictions.
5. Write or update a short implementation brief.

### Phase 1 - architecture and contracts
1. Create/update `README.md`.
2. Create/update `docs/vision.md`.
3. Create/update `docs/architecture.md`.
4. Create/update `docs/threat-model.md`.
5. Define the metacognitive state schema.
6. Define daemon/CLI/plugin boundaries.

### Phase 2 - local vertical slice
1. Scaffold the core runtime.
2. Scaffold persistence.
3. Scaffold the CLI.
4. Implement one end-to-end loop: assess -> verify -> calibrate -> compact.
5. Make it runnable locally.

### Phase 3 - Claude Code integration
1. Create the plugin manifest.
2. Add hooks.
3. Add skills.
4. Add at least one specialist agent.
5. Add plugin README and install instructions.
6. Ensure the plugin works in local `--plugin-dir` mode.

### Phase 4 - evaluation and hardening
1. Add tests.
2. Add replay fixtures.
3. Add trace logging.
4. Add benchmark scripts.
5. Add token/cost diagnostics.
6. Harden security boundaries.

### Phase 5 - optional edge standards
1. Thin MCP exposure.
2. Agent SDK adapter.
3. A2A adapter.
4. Optional WASM experiments.

## Required initial deliverables in this run

Unless the repo already contains superior equivalents, produce or update:

- `README.md`
- `CLAUDE.md`
- `docs/vision.md`
- `docs/architecture.md`
- `docs/threat-model.md`
- `docs/eval-framework.md`
- `docs/roadmap.md`
- `core/` scaffold
- `adapters/claude-plugin/` scaffold
- `adapters/claude-plugin/.claude-plugin/plugin.json`
- `adapters/claude-plugin/hooks/hooks.json`
- initial skill folders and `SKILL.md` files
- initial agent markdown files
- `adapters/claude-plugin/settings.json`
- a minimal `.mcp.json` only if justified
- developer scripts for local run/test/package

If you cannot safely implement all of these in one pass, implement the highest-value subset and leave a precise backlog.

## Working protocol

Follow this operating procedure:

1. **Inspect first, do not hallucinate the repo.**
2. **Prefer reading files and configs over making assumptions.**
3. **State architecture decisions clearly.**
4. **Make small, coherent batches of edits.**
5. **Run verification commands whenever possible.**
6. **When a direction is unclear, choose the more local, cheaper, more portable path.**
7. **When security and convenience conflict, choose security and document the UX cost.**
8. **When intelligence and token cost conflict, design a fast path and a deep path.**
9. **When raw autonomy and human control conflict, design principled escalation.**
10. **Do not drown the repo in boilerplate.**

## Output contract for this session

Your response and edits should be structured like this:

### A. Repository understanding
- what exists,
- what is missing,
- what is contradictory,
- what assumptions you made.

### B. Architecture decision summary
- chosen project shape,
- why it is correct,
- why MCP is not the core,
- what the plugin does,
- what the runtime does.

### C. Files created or changed
For each important file:
- path,
- purpose,
- key contents.

### D. Verification
- commands run,
- what passed,
- what failed,
- what remains unverified.

### E. Risks and next steps
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

## Final guardrails

Do not drift into any of the following:
- giant omniscient prompt architecture,
- MCP maximalism,
- benchmark theater without product depth,
- verbose memory pollution,
- unsafe automation,
- fake confidence metrics,
- marketplace cosmetics before core reliability,
- dependence on a single model vendor.

The final result should feel like the beginning of a **real product**:
- installable locally,
- Claude-native in UX,
- portable in architecture,
- lean in context,
- serious about verification,
- serious about safety,
- serious about human decision quality.

Start by inspecting the repository and any local research files, then execute the highest-value version of this plan immediately.
