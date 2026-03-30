# Project: Helix Agent — Fast Local Automation Stack

## Vision
The whole idea of the Helix Agent is to run local agents fast even on low-end systems, optimizing for agentic tasks with highly accurate tool calling. It acts as a local equivalent to cloud-based CLI agents (like claude-code and open-interpreter) but is engineered to run gracefully on low-end laptops using locally hosted models.

## Core Value
Speed, reliability, and precision on strictly local hardware. Users should never fight hallucinations when evaluating tools, and response latency must be minimized using optimal hardware targets.

## What This Is
A multi-layered AI orchestrator combining a Python setup/boot layer with a high-performance Rust orchestrator, talking seamlessly over localhost to an underlying `llama.cpp` or `koboldcpp` inference engine.

---

## Current Milestone: v1.2 Chat Mode Polish & Streaming Reliability

**Goal:** Chat mode produces direct, concise responses without visible reasoning. Streaming is live (token-by-token). Tool calling is non-blocking with parallel support.

**Target features:**
- Strict system prompt enforcement for chat mode (no reasoning visible)
- Intent detection to branch between chat vs. agentic modes
- Post-processing filter to strip thinking traces and clean output
- Raw byte streaming with immediate token-by-token rendering
- Non-blocking async tool execution
- Parallel & multi-tool support with concurrent execution
- Shared types crate and codebase cleanup (clippy-clean, no warnings)

---

## Completed Milestones

### v1.1 Operational Upgrades (Phases 9-14)
- ✓ Fixed terminal chat warning and optimized system prompt (Phase 9)
- ✓ Built TUI foundation with ratatui, input, and ghost autocomplete (Phase 10)
- ✓ Implemented output polish and streaming (Phase 11)
- ✓ Added control/feedback with interrupts and TTFT tracking (Phase 12)
- ✓ Added context and discoverability layers (Phase 13)
- ✓ Fixed TUI missing output bug with SSE parser repair + UAT gap closure (Phase 14)

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

- [ ] **CHAT-01 (Chat Mode Prompt):** Strict system prompt for chat mode enforcing concise, direct responses.
- [ ] **CHAT-02 (Reasoning Filter):** Post-processing layer to strip thinking traces from chat mode output.
- [ ] **STREAM-01 (Live Token Rendering):** No buffering; raw bytes read immediately and fed to terminal/TUI.
- [ ] **STREAM-02 (Immediate Redraw):** Terminal flushes after each token. TUI redraws on every chunk.
- [ ] **TOOL-01 (Non-blocking Tools):** Tool execution in async tasks without blocking UI.
- [ ] **TOOL-02 (Parallel Execution):** Multiple tool calls in one turn executed concurrently.
- [ ] **TOOL-03 (Tool Status UI):** Live status display for running tools in terminal and web UI.
- [ ] **CODE-01 (Shared Types):** Extract common types into `agent_core` crate for reuse.
- [ ] **CODE-02 (Tracing & Logging):** Structured logging for streaming delays and tool lifecycles.
- [ ] **CODE-03 (Clippy Clean):** All code passes `cargo clippy` with zero warnings.

### Out of Scope

- Cloud API endpoints handling primary cognitive loads (against the "local-first" philosophy).
- Gradio or Streamlit interfaces (decided to focus on a proper fast modern frontend framework instead).
- RAG / vector search (deferred post-v1.2).
- Multi-agent coordination (deferred post-v1.2).
- Persistent memory beyond session save/load (deferred post-v1.2).
- Full Claude-like computer use (deferred post-v1.2).

---

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Grammar Enforcement | Small 8B-14B models hallucinate JSON schemas easily. Native structural reinforcement (GBNF) forces 100% compliant outputs at the token generation level without massive context overhead. | Pending implementation across backends |
| React/Svelte Web Stack | User specified a modern frontend (Option C) over basic Python wrappers, allowing long-term flexible UX scaling for the Web UI. | Pending implementation |
| Rich Terminal Rust Crate | A primary pain-point for CLI dev tools is raw text pasting. `rustyline` resolves this seamlessly. | Pending implementation |

---
*Last updated: 2026-03-29 — Milestone v1.2 initiated*

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
