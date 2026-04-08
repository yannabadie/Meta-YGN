# Aletheia-Nexus

**Local-first safety runtime for AI coding agents.**

Aletheia-Nexus is a Rust daemon that intercepts every tool call from your AI coding agent, analyzes it through AST parsing and contextual risk scoring, creates automatic recovery checkpoints, and blocks destructive operations before they execute.

It prevents real incidents: `terraform destroy` on production, `rm -rf /`, `git push --force` without approval, and agent meltdown loops.

## Pages

- [Getting Started](Getting-Started) -- Prerequisites, installation, first run
- [Architecture](Architecture) -- 5-layer cascade, crate graph, hook lifecycle
- [CLI Reference](CLI-Reference) -- All 11 commands with flags and examples
- [API Reference](API-Reference) -- All HTTP endpoints with curl examples
- [FAQ](FAQ) -- Performance, fallback behavior, false positives, contributing
- [Troubleshooting](Troubleshooting) -- Common issues and solutions

## Quick links

- [GitHub repository](https://github.com/yannabadie/Meta-YGN)
- [Changelog](https://github.com/yannabadie/Meta-YGN/blob/master/CHANGELOG.md)
- [License (MIT)](https://github.com/yannabadie/Meta-YGN/blob/master/LICENSE)
