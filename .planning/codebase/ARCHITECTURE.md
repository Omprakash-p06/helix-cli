# Architecture

## High-Level Pattern
- Hybrid orchestrator architecture:
  - Python handles setup and local backend process management.
  - Rust handles conversation loop, tool dispatch, and guardrails.

## Main Runtime Flow
1. User launches `start.py`.
2. `start.py` prompts for model from `models/`.
3. `start.py` starts `scripts/start_server.py` with model env overrides.
4. `scripts/start_server.py` attempts llama.cpp first, then KoboldCPP fallback.
5. Once endpoint is ready, `start.py` invokes Rust orchestrator from `agent-rs/`.
6. Rust loop in `agent-rs/src/main.rs` calls model endpoint and executes tools.

## Server Layer
- Backend process is independent from orchestrator process.
- Readiness is based on HTTP polling against `/v1/models`.
- Startup diagnostics are persisted to logs in `logs/`.

## Orchestrator Layer
- Message struct and tool-call loop live in `agent-rs/src/main.rs`.
- Tool schemas and implementations live in `agent-rs/src/tools.rs`.
- Token counting and memory compaction are built into the Rust loop.

## Setup Layer
- `setup.py` performs hardware detection, dependency install, backend build, model download, and config generation.
- Context minimum and preflight gate behavior are controlled in setup.

## Failure and Fallback Behavior
- llama.cpp launch failure triggers fallback to KoboldCPP in `scripts/start_server.py`.
- Startup timeout and log-tail surfacing are handled in `start.py`.
- Eval preflight can be skipped by setup gate policy.

## Current Architectural Strengths
- Local-first design with explicit fallback path.
- Strong separation between backend serving and orchestration logic.
- Typed tool schemas in Rust reduce malformed execution risk.
