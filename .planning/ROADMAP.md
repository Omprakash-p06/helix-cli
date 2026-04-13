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
**Plans:** 2/2 plans complete complete

Plans:
- [x] 14-01-PLAN.md — Repair streaming parser and token rendering pipeline with regression tests. (completed 2026-03-29)
- [x] 14-02-PLAN.md — Close UAT gaps for stuck/blank live streaming visibility, tool-phase feedback, and interrupt flush safety. (completed 2026-03-29)

### [x] Phase 15: Chat Mode Polish Foundation

**Goal:** Implement strict system prompt isolation for chat mode and strip internal reasoning traces from user-visible output.
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04
**Depends on:** Phase 14
**Plans:** 2/2 plans complete

Plans:
- [x] 15-01-PLAN.md: Chat mode system prompt and reasoning filter (completed 2026-03-31)
- [x] 15-02-PLAN.md: Output deduplication and quote normalization (completed 2026-03-31)

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
**Plans:** 2/2 plans complete

Plans:
- [x] 17-01-PLAN.md: Async task spawning and tool status UI
- [x] 17-02-PLAN.md: Parallel execution, result ordering, and timeouts

### Phase 18: Production Quality & Codebase Consolidation

**Goal:** Extract shared types into `agent_core` crate, achieve clippy-clean status, integrate structured tracing, and build comprehensive test suite.
**Requirements**: CODE-01, CODE-02, CODE-03, CODE-04
**Depends on:** Phase 17
**Plans:** 0/1 plans

Plans:
- [ ] 18-01-PLAN.md: Types refactor, code cleanup, tracing, and tests

### [x] Phase 19: TUI redesign

**Goal:** Redesign TUI with three-panel layout, command palette, themes, ghost autocomplete, and token counter.
**Requirements**: TBD
**Depends on:** Phase 18
**Plans:** 2/2 plans

Plans:
- [x] 19-01-PLAN.md — API contract, state management, three-panel layout foundation
- [x] 19-02-PLAN.md — Command palette and theme system

### Phase 19: Implement Terminal UI features

**Goal:** Replace the placeholder TUI shell with the full Phase 19 terminal experience: configurable layout, advanced input and command palette, rich conversation/sidebar rendering, theme support, and responsive terminal feedback.
**Requirements**: TUI-01, TUI-02, TUI-03, TUI-04, TUI-05
**Depends on:** Phase 18
**Plans:** 1/4 plans executed

Plans:
- [x] 19-01-PLAN.md — Shared launch contract, layout flag parsing, and three-region TUI shell.
- [ ] 19-02-PLAN.md — Slash command palette, multiline input HUD, external editor, and theme/config loading.
- [ ] 19-03-PLAN.md — Rich conversation rendering, sidebar telemetry, preview overlays, and scroll-safe new-message behavior.
- [ ] 19-04-PLAN.md — Typing/tool progress polish, interrupt/quit UX, regression tests, and clippy-clean validation.

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

### Phase 20: Security Execution Guardrails
**Goal:** Close critical tool-execution risk by enforcing policy-tiered command execution, hardened argument validation, and prompt-injection refusal gating.
**Requirements**: SEC-01, SEC-02, SEC-03
**Depends on:** Phase 19
**Gap Closure:** Confirms and closes vulnerabilities from misc/vulnerabilities.md sections 1.1-1.3.
**Plans:** 0/3 plans complete

Plans:
- [ ] 20-01-PLAN.md — Security policy contracts and permission-tier config bridge.
- [ ] 20-02-PLAN.md — Command validation, risk scoring, and prompt-injection refusal gate.
- [ ] 20-03-PLAN.md — Runtime integration across terminal/web paths with guardrail tests.

### Phase 21: Model Integrity and Install Automation
**Goal:** Deliver trusted model supply chain checks and a one-command hardware-aware installer workflow.
**Requirements**: SEC-04, SETUP-01
**Depends on:** Phase 20
**Gap Closure:** Closes vulnerability 1.4 and market need for zero-friction model setup.
**Plans:** 0/1 plans complete

Plans:
- [ ] 21-01-PLAN.md — Trusted manifest, checksum gate, and install entrypoint wiring.

### Phase 22: Onboarding and Session Resilience
**Goal:** Reduce learning friction with guided onboarding and implement crash-safe session persistence with resume.
**Requirements**: SETUP-02, SESSION-01, SESSION-02
**Depends on:** Phase 19
**Gap Closure:** Closes vulnerabilities 2.2 and 2.4 and aligns with market expectations for professional UX.
**Plans:** 1/1 plans complete

Plans:
- [x] 22-01-PLAN.md — Onboarding profile, crash-safe autosave/resume, and slash lifecycle commands. (completed 2026-04-13)

### Phase 23: CPU TTFT and Runtime Watchdog
**Goal:** Improve perceived and sustained performance on modest hardware using CPU-aware runtime tuning and server health watchdog recovery.
**Requirements**: PERF-04, PERF-05
**Depends on:** Phase 21
**Gap Closure:** Closes vulnerabilities 3.1 and 3.2 with measurable runtime reliability improvements.
**Plans:** 0/1 plans complete

Plans:
- [ ] 23-01-PLAN.md — Runtime profile contracts, watchdog recovery policy, and cross-layer verification.

### Phase 24: Enterprise Audit Log MVP
**Goal:** Add structured, queryable audit logging for tool invocations and security decisions to support enterprise trust requirements.
**Requirements**: ENT-01
**Depends on:** Phase 20
**Gap Closure:** Closes vulnerability 4.1 (auditability baseline) and supports compliance-ready evolution.
**Plans:** 0/0 plans complete

Plans:
- [ ] TBD (run /gsd-plan-phase 24 to break down)

### Phase 25: Plugin SDK and IDE Bridge Foundation
**Goal:** Establish extensibility and IDE adoption foundations through a plugin SDK contract and local IDE integration bridge.
**Requirements**: ENT-02, IDE-01
**Depends on:** Phase 22
**Gap Closure:** Closes vulnerabilities 4.2 and 4.3 and aligns with competitive market expectations.
**Plans:** 0/0 plans complete

Plans:
- [ ] TBD (run /gsd-plan-phase 25 to break down)
