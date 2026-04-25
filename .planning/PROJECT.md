# Project: Helix OS Agent — Autonomous Local Troubleshooting Stack

## Vision
Helix OS Agent is a local-first, autonomous AI system administrator designed to troubleshoot, diagnose, and repair operating system issues entirely on consumer hardware. Powered by Qwen 3.6 and orchestrated by GSD 2.0, it provides a privacy-first, secure, and verifiable alternative to cloud-based system management tools.

## Core Value
**Safety, Autonomy, and Privacy.** Helix OS Agent bridges the gap between local AI capabilities and real-world system administration by enforcing a multi-layer security framework (Sandboxing, Policy Enforcement, Audit Logging) while delivering state-of-the-art agentic performance on strictly local hardware.

## What This Is
A defense-in-depth AI orchestrator combining a high-performance Rust core (`agent-rs`) with an isolated execution sandbox. It utilizes the GSD 2.0 protocol to manage complex, multi-step troubleshooting workflows, ensuring every action is planned, verified, and auditable.

---

## Current Milestone: v2.0 Helix OS Agent Pivot

**Goal:** Transform the general-purpose coding assistant into a specialized OS troubleshooting agent with a robust security foundation.

**Target features:**
- Qwen 3.6 integration (27B/35B MoE) for superior terminal task performance.
- Docker/MicroVM execution sandboxing for all agent operations.
- Command canonicalization and strict allowlist policy engine.
- GSD 2.0 Pi SDK integration for autonomous orchestration.
- Human-in-the-loop approval gates for system modifications.
- Immutable, append-only audit logging for compliance and forensics.

---

## Completed Milestones (Legacy Helix Agent)

### v1.1 Operational Upgrades
- ✓ Built TUI foundation with ratatui and rich input (Phases 10-14)
- ✓ Implemented streaming and control feedback (Phases 11-12)
- ✓ Grammar-enforced tool calling foundation (Phase 3)

---

## Requirements

### Validated Foundation

- ✓ Local LLM inference backend wrapper (llama.cpp/koboldcpp)
- ✓ Rust orchestrator loop with TUI/Web UI support
- ✓ Deterministic Grammar-Enforced Tool Calling (GBNF)
- ✓ Multi-mode execution (Agentic / Chat)

### Active (Pivot Phase 01)

- [ ] **SEC-01 (Execution Sandbox):** Build Docker/MicroVM wrapper for all agent commands.
- [ ] **SEC-02 (Policy Engine):** Implement command canonicalization and strict allowlist.
- [ ] **SEC-03 (Audit Log):** Implement append-only immutable audit logging.
- [ ] **MOD-01 (Qwen 3.6):** Integrate Qwen 3.6 tiered model loading via llama.cpp.

### Active (Pivot Phase 02-03)

- [ ] **DIAG-01 (OS Introspection):** Cross-platform log readers and system state tools.
- [ ] **FIX-01 (Approval Gate):** Mandatory "Ask-for-Permission" flow for repair actions.
- [ ] **FIX-02 (Rollback):** Pre-repair filesystem/state snapshots.

### Active (Pivot Phase 04-05)

- [ ] **GSD-01 (Orchestration):** GSD 2.0 Pi SDK integration for multi-phase planning.
- [ ] **GSD-02 (Autonomous Recovery):** RETRY/DECOMPOSE/PRUNE logic for failed tasks.
- [ ] **SEC-04 (Guardian):** Multi-agent voting consensus for high-risk actions.

---

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Qwen 3.6 Foundation | Matches/exceeds cloud models on Terminal-Bench 2.0 while fitting on consumer VRAM. | Strategic Foundation |
| GSD 2.0 Orchestration | Provides autonomous error recovery, context reset, and verifiable outcomes. | Orchestration Standard |
| Sandbox-First Security | AI agents with shell access are a security nightmare; architectural boundaries must exist. | Non-negotiable Requirement |
| Local-First Privacy | OS troubleshooting involves sensitive logs and system state; cloud APIs are a privacy risk. | Core Value |

---
*Last updated: 2026-04-24 — Pivot to Helix OS Agent initiated*
