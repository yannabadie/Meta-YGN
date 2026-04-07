---
title: WASM Sandbox
type: feature
evidence_tier: experimental
crate: metaygn-sandbox
tags: [feature, experimental, sandbox, wasm]
created: 2026-04-07
---

# WASM Sandbox

Tier 4 capability-based isolation in the ADR-004 cascade verification pipeline. Executes untrusted code in a sandboxed Wasmtime environment with strict resource limits.

## How it works

- Wasmtime v43 runtime with deny-by-default capability model
- Fuel-based execution timeout (configurable, default prevents infinite loops)
- Memory limits enforced by Wasmtime engine configuration
- Currently executes WAT (WebAssembly Text) modules

## Security model

- **Deny-by-default**: no filesystem, network, or environment access unless explicitly granted
- **Fuel-limited**: execution halted when fuel budget exhausted
- **Memory-bounded**: configurable memory ceiling per module
- **Isolated**: each execution gets a fresh store, no cross-invocation state leakage

## Feature gate

- `--features wasm`
- When disabled, Tier 4 is skipped entirely

## Daemon endpoint

- `POST /sandbox/wasm` accepts WAT module source and returns execution results (stdout/stderr, fuel consumed, exit status)

## Roadmap

- WASI stdout/stderr capture planned for v2.4
- Pre-compiled module caching
- Capability grants for controlled filesystem access

## References

- ADR-004 (cascade verification architecture)
- Wasmtime capability-based security model
- Principle of least privilege (deny-by-default)
