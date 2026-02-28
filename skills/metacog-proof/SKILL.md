---
name: metacog-proof
description: Build a proof packet before finalizing work. Use for risky edits, architecture claims, dependency changes, security-sensitive work, and any answer that could mislead a reviewer.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools: Read, Grep, Glob, Bash
---

# Proof Packet

Build the smallest proof packet that a skeptical reviewer would trust.

Separate the result into:

## Goal
State the task in one sentence.

## Claim
What is being asserted (code works, design is sound, migration is safe, etc.).

## Evidence gathered
List concrete evidence: files read, patterns found, docs checked.

## Checks run
List verification commands executed and their results:
- tests: command + pass/fail
- lint/type check: command + result
- build: command + result
- grep/search: pattern + findings

## What remains unverified
Honestly list what could not be checked with available tools.

## Recommended next step
The single smallest action that would increase confidence or reduce risk.

---

**Rules:**
- Prefer independent checks over restating the draft answer.
- If no meaningful check was run, say so plainly.
- If the risk is high and evidence is weak, recommend escalation or a narrower change.
- Tag each evidence item as `[confirmed]`, `[experimental]`, or `[unverified]`.
