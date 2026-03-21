# Project: Helix Agent — Fast Local Automation Stack

## Vision
The whole idea of the Helix Agent is to run local agents fast even on low-end systems, optimizing for agentic tasks with highly accurate tool calling. It acts as a local equivalent to cloud-based CLI agents (like claude-code and open-interpreter) but is engineered to run gracefully on low-end laptops using locally hosted models.

## Core Value
Speed, reliability, and precision on strictly local hardware. Users should never fight hallucinations when evaluating tools, and response latency must be minimized using optimal hardware targets.

## What This Is
A multi-layered AI orchestrator combining a Python setup/boot layer with a high-performance Rust orchestrator, talking seamlessly over localhost to an underlying `llama.cpp` or `koboldcpp` inference engine.

---

## Requirements

### Validated

- ✓ Local LLM inference backend wrapper (llama.cpp)
- ✓ Fallback backend support (koboldcpp)
- ✓ Python hardware benchmarking and unified setup script
- ✓ Rust orchestrator loop with local filesystem/terminal tool access

### Active

- [ ] **Dual UI Launcher:** `start.py` must prompt users to select between (1) Web Interface or (2) Terminal Chat.
- [ ] **Dual Mode Execution:** `start.py` must support selecting between 'Agentic' (tool-equipped) and 'Chat' modes.
- [ ] **Cross-Platform Parity:** Ensure identical standards, paths, and execution behaviors across Windows, Linux, and Arch Linux.
- [ ] **Clinical Agent Personas:** Eliminate all LLM personality trails, conversational filler, and greeting loops (e.g., "Hello! I'm your OS Assistant"). The agent must be purely functional, concise, and focused on tool-calling.
- [ ] **Modern Web Interface:** A lightweight, modern JS framework (React/Vue/Svelte) frontend communicating with the orchestration backend APIs.
- [ ] **Rich Terminal Input:** Upgrade the Rust CLI to gracefully support pasting multi-line strings/code using crates like `rustyline` or `inquire`.
- [ ] **Deterministic Tool Calling (llama.cpp):** Enforce strict JSON output matching the Rust tool schemas natively by generating GBNF (GGML BNF) Grammar dynamically on the fly.
- [ ] **Deterministic Tool Calling (koboldcpp):** Implement parallel structural or grammar logic for the koboldcpp fallback to ensure reliable tool use.

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
