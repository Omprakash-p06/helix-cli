# Project: Helix Agent — Fast Local Automation Stack

## Vision
The whole idea of the Helix Agent is to run local agents fast even on low-end systems, optimizing for agentic tasks with highly accurate tool calling. It acts as a local equivalent to cloud-based CLI agents (like claude-code and open-interpreter) but is engineered to run gracefully on low-end laptops using locally hosted models.

## Core Value
Speed, reliability, and precision on strictly local hardware. Users should never fight hallucinations when evaluating tools, and response latency must be minimized using optimal hardware targets.

## What This Is
A multi-layered AI orchestrator combining a Python setup/boot layer with a high-performance Rust orchestrator, talking seamlessly over localhost to an underlying `llama.cpp` or `koboldcpp` inference engine.

---

## Current Milestone: v1.1 Operational Upgrades

**Goal:** Improve inference reliability, hardware resource utilization, agent transparency, and developer UX.

**Target features:**
- Clear GPU memory (kill lingering processes) before booting agents.
- Expose agent thoughts using `<thinking></thinking>` blocks.
- Create automated testing for agentic tool call accuracy.
- Prioritize fast TTFT and generation speed.
- Implement dGPU to iGPU fallback if VRAM is exhausted.
- Fix terminal input UX so Enter-on-empty is replaced with a standard submission.

---

## Requirements

### Validated

- ✓ Local LLM inference backend wrapper (llama.cpp)
- ✓ Fallback backend support (koboldcpp)
- ✓ Python hardware benchmarking and unified setup script
- ✓ Rust orchestrator loop with local filesystem/terminal tool access
- ✓ Dual UI Launcher (Terminal / Web)
- ✓ Dual Mode Execution (Agentic / Chat)
- ✓ Clinical Agent Personas without hallucinated dialogue
- ✓ Modern Web Interface React app with real-time SSE streaming
- ✓ Rich Terminal Input with Bracketed Paste
- ✓ Deterministic Grammar-Enforced Tool Calling (GBNF schemas) across backends

### Active

- [ ] **PERF-01 (GPU Memory Clearing):** Aggressively clean VRAM/lingering engine processes before any boot.
- [ ] **PERF-02 (Fast Responses):** Re-tune system args/context configs to prioritize highest tokens/sec.
- [ ] **PERF-03 (iGPU Fallback):** Automatically transition to iGPU or Unified Memory if the dedicated GPU fails to allocate the model.
- [ ] **UX-01 (Visible Thoughts):** Map internal `<think>` blocks to visible `<thinking></thinking>` UI segments.
- [ ] **UX-02 (Terminal Input):** Remove the "double enter" submission logic in terminal in favor of standard single-Enter or Ctrl+D overrides for better UX.
- [ ] **TEST-01 (Accuracy Profiling):** Build a dedicated script to test tool schemas against local models programmatically.

### Out of Scope

- Cloud API endpoints handling primary cognitive loads (against the "local-first" philosophy).
- Gradio or Streamlit interfaces (decided to focus on a proper fast modern frontend framework instead).

---

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Grammar Enforcement | Small 8B-14B models hallucinate JSON schemas easily. Native structural reinforcement (GBNF) forces 100% compliant outputs at the token generation level without massive context overhead. | Pending implementation across backends |
| React/Svelte Web Stack | User specified a modern frontend (Option C) over basic Python wrappers, allowing long-term flexible UX scaling for the Web UI. | Pending implementation |
| Rich Terminal Rust Crate | A primary pain-point for CLI dev tools is raw text pasting. `rustyline` resolves this seamlessly. | Pending implementation |

---
*Last updated: 2026-03-21 after initialization*

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state
