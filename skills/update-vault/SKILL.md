---
name: update-vault
description: Synchronize the Obsidian vault (Meta-YGN/) with the current codebase state. Use after releases, major changes, or when vault docs may be stale.
user-invocable: true
disable-model-invocation: false
allowed-tools: Read, Write, Edit, Grep, Glob, Bash, Agent
---

# Update Obsidian Vault

Synchronize the Obsidian documentation vault at `Meta-YGN/` with the current state of the Aletheia-Nexus codebase.

## When to use

- After a version bump or release tag
- After merging a feature branch with significant changes
- When the user asks to update documentation
- Periodically to prevent drift

## Process

### 1. Gather current state

Read these sources of truth:
- `Cargo.toml` — current version
- `CHANGELOG.md` — latest changes
- `README.md` — what's confirmed vs experimental
- `.claude-plugin/plugin.json` — plugin version
- `crates/*/src/` — actual module structure
- `crates/*/tests/` — test coverage
- `git log --oneline -20` — recent commits

### 2. Read the vault dashboard

Read `Meta-YGN/00-Dashboard.md` and compare:
- Version number
- Feature status table (confirmed vs experimental)
- Key metrics (test count, LOC, crate count)
- Limitations section

### 3. Update each section

For each vault file that is out of date:

**Dashboard** (`00-Dashboard.md`):
- Version, feature tiers, metrics, limitations

**Architecture files** (`Architecture/*.md`):
- Endpoint count in Daemon.md
- Guard rules count in Guard-Pipeline.md
- Any new architectural components

**Features** (`Features/*.md`):
- Promote features from experimental to confirmed when tests exist
- Create new feature pages for features added since last sync
- Update evidence tables with new test coverage

**Decisions** (`Decisions/*.md`):
- Add new ADRs if architectural decisions were made

### 4. Verify vault integrity

- Check all `[[wikilinks]]` resolve to existing files
- Ensure frontmatter `updated` fields are set to today
- Verify no broken links in MOC (Map of Content) files

### 5. Commit

```bash
git add Meta-YGN/
git commit -m "docs(vault): sync Obsidian vault with codebase vX.Y.Z"
```

## What NOT to update

- Templates (`Templates/*.md`) — only change if the template structure needs updating
- Session reviews — these are historical records, not living docs
- ADR status — only change if a decision is actually reversed

## Output

Report:
- Files updated (with summary of changes)
- New files created
- Features promoted (experimental -> confirmed)
- Remaining gaps (if any)
