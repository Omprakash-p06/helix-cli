# Roadmap

## [x] Phase 1: Boot Upgrades & Mode Selection
**Goal:** Enhance the single-command launcher (`start.py`) to provide the necessary UX prompts for UI and Agentic mode selection, and strip existing personality configurations.
*   **Requirements:** REQ-04, REQ-05, REQ-09, REQ-10
*   **Success Metrics:** Executing `start.py` visibly pauses for user input, successfully captures variables for Web/Terminal and Agentic/Chat modes, and passes these variables gracefully into the Rust orchestrator environment. The system prompt natively forbids greetings.

## [x] Phase 2: Rich Terminal Upgrades
**Goal:** Replace the naive standard input loop in `agent-rs/src/main.rs` with a robust text-area handling crate.
*   **Requirements:** REQ-07
*   **Success Metrics:** A user can copy a 50-line code snippet, paste it into the Rust terminal prompt, and naturally submit the entire block without the terminal fracturing the input lines or dropping characters.

## [x] Phase 3: Grammar-Enforced Tool Calling
**Goal:** Ensure 100% JSON schema compliance natively at the token generation layer for local models.
*   **Requirements:** REQ-06
*   **Success Metrics:** The Rust orchestrator automatically converts `tools::ToolCallArgs` into a valid GBNF grammar string and successfully passes it as an enforced constraint to the `llama-server` `/v1/chat/completions` API endpoint during agentic turns. Hallucinated schemas drop to zero.

## [x] Phase 4: KoboldCPP Fallback Accuracy
**Goal:** Ensure the fallback inference backend achieves the same exact tool-calling accuracy.
*   **Requirements:** REQ-06
*   **Success Metrics:** If the system is booted using `koboldcpp`, the engine is correctly invoked with accurate structural formatting (or API-compatible grammar equivalents) mimicking Phase 3's reliability.

## [x] Phase 5: Modern Web Frontend
**Goal:** Build the alternative "Web Interface" option requested during the boot sequence.
*   **Requirements:** REQ-08
*   **Success Metrics:** A standalone web application (React/Vue/Svelte) can boot simultaneously, connect to the local server, display streaming tool executions, and handle user chat seamlessly.

### [x] Phase 9: Fix terminal chat warning and optimize system prompt
**Goal:** Remove `unused_mut` compiler warning and suppress the default terminal chat system prompt to optimize Time-To-First-Token.
**Requirements**: UX-03 (Performance Harmonization)
**Depends on:** Phase 8
**Plans:** 1/1 plans complete

Plans:
- [x] 01-PLAN.md: Fix warning and optimize prompt push.

### [x] Phase 10: Terminal UI Foundation

**Goal:** Implement foundational TUI using ratatui, crossterm, and tokio with input handling, chat rendering, ghost autocomplete, and status bar.
**Requirements**: TBD
**Depends on:** Phase 9
**Plans:** 1/1 plans complete

Plans:
- [x] 01-PLAN.md: Ratatui foundation, input layer, multiline, command preview.

### Phase 11: Output Polish and Streaming

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 10
**Plans:** 1/1 plans complete

Plans:
- [x] TBD (run /gsd-plan-phase 11 to break down) (completed 2026-03-29)

### Phase 12: Control and Feedback

**Goal:** Mid-stream generation interrupts (Ctrl+C), chat history scrolling (PageUp/Down), and live TTFT tracking in the TUI.
**Requirements**: UX-01, UX-02, PERF-02
**Depends on:** Phase 11
**Plans:** 0/1 plans complete

Plans:
- [x] 12-01-PLAN.md: Interactive Controls and Telemetry

### Phase 13: Context and Discoverability

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 12
**Plans:** 0/0 plans complete

Plans:
- [x] TBD (run /gsd-plan-phase 13 to break down) (completed 2026-03-29)

### Phase 14: Fix TUI Missing Output Bug

**Goal:** Restore reliable TUI output by fixing streamed SSE parsing and token-to-UI rendering flow.
**Requirements**: UX-01, TEST-01
**Depends on:** Phase 13
**Plans:** 2/2 plans complete

Plans:
- [x] 14-01-PLAN.md — Repair streaming parser and token rendering pipeline with regression tests. (completed 2026-03-29)
- [x] 14-02-PLAN.md — Close UAT gaps for stuck/blank live streaming visibility, tool-phase feedback, and interrupt flush safety. (completed 2026-03-29)

### Phase 15: Chat Mode Polish Foundation

**Goal:** Implement strict system prompt isolation for chat mode and strip internal reasoning traces from user-visible output.
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04
**Depends on:** Phase 14
**Plans:** 0/2 plans

Plans:
- [ ] 15-01-PLAN.md: Chat mode system prompt and reasoning filter
- [ ] 15-02-PLAN.md: Output deduplication and quote normalization

### Phase 16: Live Streaming & Immediate Rendering

**Goal:** Refactor SSE streaming from line-buffered to byte-level reads; eliminate accumulation delays; ensure immediate token rendering.
**Requirements**: STREAM-01, STREAM-02, STREAM-03, STREAM-04, STREAM-05
**Depends on:** Phase 15
**Plans:** 0/2 plans

Plans:
- [ ] 16-01-PLAN.md: Byte-level SSE parsing and immediate render
- [ ] 16-02-PLAN.md: Terminal and TUI reactivity, interrupt safety

### Phase 17: Non-Blocking Tool Execution

**Goal:** Implement async tool spawning, concurrent execution, and status feedback without blocking the orchestrator loop.
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-05
**Depends on:** Phase 16
**Plans:** 0/2 plans

Plans:
- [ ] 17-01-PLAN.md: Async task spawning and tool status UI
- [ ] 17-02-PLAN.md: Parallel execution, result ordering, and timeouts

### Phase 18: Production Quality & Codebase Consolidation

**Goal:** Extract shared types into `agent_core` crate, achieve clippy-clean status, integrate structured tracing, and build comprehensive test suite.
**Requirements**: CODE-01, CODE-02, CODE-03, CODE-04
**Depends on:** Phase 17
**Plans:** 0/1 plans

Plans:
- [ ] 18-01-PLAN.md: Types refactor, code cleanup, tracing, and tests

---

---

## [x] Phase 6: Memory Management & Fallback
**Goal:** Optimize hardware resource utilization by aggressively clearing VRAM and implementing iGPU fallback.
*   **Requirements:** PERF-01, PERF-03
*   **Success Metrics:** 
    1. `start.py` detects and kills orphan `llama-server` or `koboldcpp` processes before launch.
    2. Fallback logic automatically routes model loading to the iGPU if dGPU VRAM allocation fails.

## [x] Phase 7: Generation Speed & Thought UI
**Goal:** Maximize TTFT (Time-To-First-Token) and expose agent reasoning visually to the user.
*   **Requirements:** PERF-02, UX-01
*   **Success Metrics:**
    1. Inference configurations are tuned for maximum tokens/sec.
    2. Internal `<think>` tags are preserved and beautifully rendered as `<thinking>` blocks in both Terminal and Web UI.

## [x] Phase 8: Terminal Input UX & Testing
**Goal:** Overhaul the terminal submission experience and build the automated tool-accuracy evaluation script.
*   **Requirements:** UX-02, TEST-01
*   **Success Metrics:**
    1. Terminal handles multi-line paste intuitively, submitting on standard Enter or Ctrl+D without requiring double-enter.
    2. A dedicated test script programmatically validates tool-call structural accuracy against backend models.
