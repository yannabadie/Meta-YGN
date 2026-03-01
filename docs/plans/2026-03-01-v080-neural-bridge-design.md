# v0.8.0 "Neural Bridge" — Design Document

**Date**: 2026-03-01
**Status**: Approved

## Goal
Connect MetaYGN to the MCP ecosystem, enable real neural embeddings for semantic memory, and add session replay for debugging.

## 3 Features

### 1. MCP Bridge (crate mcp-bridge)
New crate using rmcp 0.17 (official Rust MCP SDK). Stdio server exposing 5 metacognitive tools. Communicates with daemon via HTTP localhost. CLI command `aletheia mcp` launches the server.

Tools: metacog_classify, metacog_verify, metacog_recall, metacog_status, metacog_prune.

### 2. fastembed (feature-gated)
Real neural embeddings via fastembed 5.x with bge-small-en-v1.5 (384 dim). Feature-gated behind `cargo build --features embeddings`. HashEmbedProvider remains the default. GraphMemory generates embeddings on node insert + semantic_search method.

### 3. Session replay
Timeline recording of all hook calls + responses in SQLite. CLI `aletheia replay <id>` for post-session review. Endpoints: GET /replay/sessions, GET /replay/{id}.
