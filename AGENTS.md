# MetaYGN Codex Guardrails

Use this workflow when operating in Codex with the `aletheia` MCP server.

## Session Protocol

1. At task start, call `aletheia.metacog_classify` with the user request.
2. Before risky actions (destructive shell, auth/security/data changes), call `aletheia.metacog_status`.
3. After meaningful execution or edit batches, call `aletheia.metacog_verify`.
4. If error loops appear (3+ repeated failures), call `aletheia.metacog_prune` with recent assistant/tool messages and follow the injected recovery hint.
5. Before final answer, call `aletheia.metacog_status` and include a short risk/evidence note.

## Strict Gate (Default)

- Do not send a completion-style final answer unless both checks were called in this session after the latest meaningful action:
- `aletheia.metacog_verify`
- `aletheia.metacog_status`
- If one check is missing, first state: `META-YGN BLOCK: verification gate not satisfied; running required checks.`
- Then run the missing checks and only after that deliver the final answer.

## Minimum Reporting In Final Reply

- `Risk`: low/medium/high from the latest metacognitive signal.
- `Evidence`: key command/test/tool outputs that justify completion.
- `Uncertainty`: what is still not verified.
