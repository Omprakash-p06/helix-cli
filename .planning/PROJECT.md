# Project: Helix OS Agent — Autonomous Local Troubleshooting Stack

## Vision
Helix OS Agent is a local-first, autonomous AI system administrator designed to troubleshoot, diagnose, and repair operating system issues entirely on consumer hardware. It is a **multi-model orchestrator** capable of running any local LLM (GGUF) provided in the `models/` folder or downloaded from Hugging Face. Orchestrated by GSD 2.0, it provides a privacy-first, secure, and verifiable alternative to cloud-based system management tools.

## Core Value
**Safety, Autonomy, and Privacy.** Helix OS Agent bridges the gap between local AI capabilities and real-world system administration by enforcing a multi-layer security framework (Sandboxing, Policy Enforcement, Audit Logging) while delivering state-of-the-art agentic performance on strictly local hardware, regardless of the underlying model.

## What This Is
A defense-in-depth AI orchestrator combining a high-performance Rust core (`agent-rs`) with an isolated execution sandbox. It utilizes the GSD 2.0 protocol to manage complex, multi-step troubleshooting workflows, ensuring every action is planned, verified, and auditable.

---

## Current Milestone: v2.0 Helix OS Agent Pivot

**Goal:** Transform the general-purpose coding assistant into a specialized, multi-model OS troubleshooting agent with a robust security foundation.

**Target features:**
- **Dynamic Model Selection:** Automatically detect and allow users to choose from any model in the `models/` directory.
- **Hugging Face Integration:** Download any compatible GGUF model directly into the local environment.
- **Docker/MicroVM Sandboxing:** Isolated execution for all agent operations.
- **GSD 2.0 Pi SDK Integration:** Autonomous orchestration and UI-level command suggestions (autofill).
- **Human-in-the-loop Approval:** Safety gates for all system modifications.
- **Immutable Audit Logging:** Append-only logs for compliance and forensics.

---

## Completed Milestones (Legacy & Foundation)

### v1.1 Operational Upgrades
- ✓ Built TUI foundation with ratatui and rich input (Phases 10-14)
- ✓ Implemented streaming and control feedback (Phases 11-12)
- ✓ Grammar-enforced tool calling foundation (Phase 3)

### v2.0 Pivot Progress
- ✓ Phase 01: Multi-model foundation loader and security sandbox.
- ✓ Phase 02: Cross-platform OS diagnostics and DRL engine.
- ✓ Phase 03: HITL safety gates and transactional repairs.
- ✓ Phase 04: GSD 2.0 orchestration and recovery operators.

---

## Requirements

### Validated Foundation

- ✓ Local LLM inference backend wrapper (llama.cpp/koboldcpp)
- ✓ Rust orchestrator loop with TUI/Web UI support
- ✓ Deterministic Grammar-Enforced Tool Calling (GBNF)
- ✓ Multi-mode execution (Agentic / Chat)

### Active (Pivot Hardening)

- [ ] **MOD-02 (Model Management):** Dynamic model discovery and selection UI.
- [ ] **MOD-03 (HF Downloader):** Tool for downloading models from Hugging Face.
- [ ] **UX-01 (GSD Autofill):** Automatic GSD command suggestions in the input field.
- [ ] **SEC-05 (Blocklist):** Hardcoded non-bypassable destructive command blocklist.

---

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Multi-Model Support | Vision is tool-centric, not model-centric. Users should use the best local model for their hardware. | Strategic Pivot |
| GSD 2.0 Orchestration | Provides autonomous error recovery, context reset, and verifiable outcomes. | Orchestration Standard |
| Sandbox-First Security | AI agents with shell access are a security nightmare; architectural boundaries must exist. | Non-negotiable Requirement |
| Local-First Privacy | OS troubleshooting involves sensitive logs and system state; cloud APIs are a privacy risk. | Core Value |

---
*Last updated: 2026-04-26 — Generalized to multi-model vision and removed persistent sessions.*
