---
title: OpenTelemetry Exporter
type: feature
evidence_tier: confirmed
crate: metaygn-daemon
tags:
  - feature
  - confirmed
  - observability
created: 2026-04-07
---

# OpenTelemetry OTLP Exporter

**Tier**: `[confirmed]`
**Crate**: `metaygn-daemon`
**Fichier principal**: `crates/daemon/src/telemetry.rs`

## Description

Exporte les spans de tracing vers un collecteur OTLP (Jaeger, Grafana Tempo, etc.)
via gRPC/tonic. Feature-gated derriere `--features otel`.

## Implementation

- **Module**: `telemetry.rs` — initialisation conditionnelle
- **Sans otel**: `tracing_subscriber::fmt` standard
- **Avec otel**: fmt layer + OpenTelemetryLayer avec OTLP SpanExporter
- **Config**: `OTEL_EXPORTER_OTLP_ENDPOINT` (default: `http://localhost:4317`)
- **Feature gate**: oui (`--features otel`)

## Evidence

| Type | Detail | Resultat |
|------|--------|----------|
| Build sans feature | cargo build -p metaygn-daemon | pass |
| Build avec feature | cargo build --features otel | pass |
| Tests | cargo test -p metaygn-daemon | pass (127 tests) |

## Limitations connues

- Pas de metriques OTEL (seulement traces/spans)
- Prometheus /metrics est un endpoint separe (pas via OTEL)
