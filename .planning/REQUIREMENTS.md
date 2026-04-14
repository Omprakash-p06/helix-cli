# Requirements

## Core Objective
Deliver a local-first, privacy-respecting AI automation stack that runs fast on low-end hardware while matching the tool-calling accuracy of cloud agents like Claude.

## Validated Core Features
*   **REQ-01: Local Inference Backends** - The system must wrap and manage `llama.cpp` natively, while retaining a robust fallback to `koboldcpp` if compilation fails or hardware lacks support.
*   **REQ-02: Isolated Orchestrator** - A Rust executable (`agent-rs`) must handle the cognitive loops, memory compaction token limits, and local system tool execution completely isolated from the inference engine.
*   **REQ-03: Boot Lifecycle Mapping** - `setup.py` and `start.py` must control hardware detection, prompt for model downloads natively, and synchronize the two processes via localhost port 8080.

## New Feature Requirements (Active)

### REQ-04: Dual UI Launcher Prompt
When the user executes `start.py`, before launching the Rust orchestrator, the CLI must prompt the user with two distinct options:
1. Web Interface
2. Terminal Chat

### REQ-05: Dual Execution Modes
Alongside the UI prompt, `start.py` must allow the user to select the behavioral mode:
1. Agentic Mode (autonomous tool-calling enabled)
2. Chat Mode (pure chat, no autonomous tool execution)

### REQ-06: Accurate Tool Calling via Grammar
Small 8B-14B local models frequently hallucinate JSON schema formats. To counteract this:
*   **llama.cpp:** The Rust orchestrator must dynamically convert its active JSON Schema tools into GBNF (GGML BNF) Grammar and enforce this via the backend API.
*   **koboldcpp:** Implement structural enforcement or rely on specific prompt engineering formatting if native grammar support differs for the fallback backend.

### REQ-07: Rich Terminal Input
The overarching Rust terminal loop currently uses standard stdin. It must be upgraded to support a rich terminal text area (e.g., using `rustyline` or `inquire`) that inherently supports pasting multi-line code/logs without breaking the input buffer or submitting prematurely.

### REQ-08: Modern Web Frontend
Design and build a lightweight, fast modern Javascript frontend (React/Vue/Svelte) that connects to the underlying local Orchestrator/API. It must be decoupled from the core terminal loop but provide the exact same agentic capabilities.

### REQ-09: Cross-Platform Parity
The execution environments, terminal commands, and python boot scripts must operate with identical reliability across Windows, generic Linux distributions, and specifically Arch Linux. Pathing and dependency resolutions must be fully OS-agnostic.

### REQ-10: Clinical Persona Enforcement
Completely strip all "helpful assistant" personality traits, greetings, and conversational loops from the LLM outputs. The orchestrator must enforce strict, concise, and purely functional responses. The agent should only output its reasoning and execute tools without pleasantries.

## Out of Scope
*   Handling external cloud LLM APIs (e.g., Anthropic, OpenAI) for the primary loop.
*   Building UI prototypes with Gradio/Streamlit (as rejected in favor of a modern JS framework).

---

## Milestone v1.1 Requirements (Completed Phases 9-14)

### Performance & Hardware (PERF)
- [x] **PERF-01**: The launcher must automatically clear lingering GPU memory (e.g., orphaned `llama-server` or `koboldcpp` processes) before booting a new agent.
- [x] **PERF-02**: The inference pipeline must prioritize fast Time-To-First-Token (TTFT) and high generation speed via configuration tuning.
- [x] **PERF-03**: The system must implement a fallback mechanism to route inference to the iGPU if the primary dGPU VRAM is exhausted.

### User Experience (UX)
- [x] **UX-01**: The orchestrator must extract internal `<think>` blocks and expose them visibly in the terminal and web UI as format-friendly `<thinking>...</thinking>` segments.
- [x] **UX-02**: The terminal input loop must provide an intuitive submission method (e.g., Enter for send, Shift+Enter for newline) rather than requiring users to press Enter on an empty line.

### Testing & Validation (TEST)
- [x] **TEST-01**: The repository must include an automated evaluation script to explicitly test local model tool-calling accuracy against the predefined schemas.

---

## Milestone v1.2 Requirements (Active)

### Chat Mode Sanity (CHAT)
- [x] **CHAT-01**: Strict system prompt enforcement for chat mode that forbids visible reasoning, internal deliberation, and produces direct, concise responses.
- [x] **CHAT-02**: Post-processing filter that removes all `<think>`, `<thinking>`, `<analysis>`, and similar internal reasoning markers before display.
- [x] **CHAT-03**: Deduplication of consecutive identical phrases/sentences to clean up repeated model outputs.
- [x] **CHAT-04**: Normalization of quotes, backticks, and markdown artifacts to ensure professional formatting.

### Live Streaming (STREAM)
- [ ] **STREAM-01**: Raw byte-level reading from SSE stream (not line-buffered) to enable immediate token capture.
- [ ] **STREAM-02**: Immediate token rendering with no accumulation delay (remove 30ms timer from v1.1).
- [ ] **STREAM-03**: Terminal mode stdout flush after each token chunk for live visual feedback.
- [ ] **STREAM-04**: TUI redraw triggered on every token event (no throttling, real-time responsiveness).
- [ ] **STREAM-05**: Interrupt-safe token preservation where Ctrl+C displays partial accumulated output instead of blank.

### Non-blocking Tool Execution (TOOL)
- [ ] **TOOL-01**: Tool execution spawned as async tokio tasks, returning immediately without blocking orchestrator loop.
- [ ] **TOOL-02**: Tool status feedback displayed in chat UI area as "tool_name: running..." during execution.
- [ ] **TOOL-03**: Tool result injected into chat history as synthetic ChatMessage::ToolResult after completion.
- [ ] **TOOL-04**: Multiple tool calls in single response executed concurrently (parallel execution).
- [ ] **TOOL-05**: Individual tool timeout enforcement (30s max per tool) to prevent hung execution.

### Codebase Cleanup & Quality (CODE)
- [ ] **CODE-01**: Extract common types into dedicated `agent_core` crate for reuse across orchestrator and CLI.
- [ ] **CODE-02**: All code passes `cargo clippy` with zero warnings and adheres to Rust formatting standards.
- [ ] **CODE-03**: Structured tracing integration to log streaming delays, token rates, and tool lifecycle events.
- [ ] **CODE-04**: Comprehensive test suite covering chat filtering edge cases, streaming UTF-8 robustness, and tool result ordering.

## Traceability
*Will be mapped by the roadmapper to phases and success criteria.*

---

## Milestone v1.3 Requirements (Gap Closure: Security, UX, and Market Fit)

### Security & Trust (SEC)
- [ ] **SEC-01**: Enforce command execution policy tiers (read-only, workspace-write, full) with explicit per-command approval for high-risk operations.
- [ ] **SEC-02**: Validate and normalize all tool-call arguments with schema-level checks and command argument sanitization before execution.
- [ ] **SEC-03**: Add prompt-injection resistance policy with pre-execution refusal checks for destructive or exfiltration-intent actions.
- [ ] **SEC-04**: Add model integrity verification (SHA-256 checksum allowlist + trusted source metadata) before model activation.

### Setup & Onboarding (SETUP)
- [ ] **SETUP-01**: Provide one-command model install workflow (`helix install <model>`) that auto-selects quantization for detected hardware.
- [ ] **SETUP-02**: Provide guided first-run onboarding (mode/interface quick tour and safe defaults) in CLI/TUI.

### Session Reliability (SESSION)
- [ ] **SESSION-01**: Implement automatic session persistence with crash-safe autosave and startup resume prompt.
- [ ] **SESSION-02**: Implement explicit session lifecycle commands (`/save`, `/load`, `/resume`) with verified persistence backend.

### Performance & Runtime Reliability (PERF)
- [ ] **PERF-04**: Add hardware-aware runtime profile selection for CPU-only systems to reduce TTFT and avoid perceived freeze.
- [ ] **PERF-05**: Add inference watchdog (memory/health monitor + controlled restart policy) for long-running stability.

### Enterprise & Extensibility (ENT)
- [ ] **ENT-01**: Implement structured, queryable audit logs for every tool call and policy decision (timestamp, actor, inputs hash, outcome).
- [ ] **ENT-02**: Provide plugin/tool SDK with registration, permission declaration, and sandbox boundary contracts.

### Workflow Reach (IDE)
- [ ] **IDE-01**: Provide IDE integration foundation (local API contract + extension bridge spec) for in-editor usage.

## v1.3 Traceability Matrix

| Requirement ID | Phase | Status |
|----------------|-------|--------|
| SEC-01 | 20-security-execution-guardrails | Pending |
| SEC-02 | 20-security-execution-guardrails | Pending |
| SEC-03 | 20-security-execution-guardrails | Pending |
| SEC-04 | 21-model-integrity-and-install-automation | Pending |
| SETUP-01 | 21-model-integrity-and-install-automation | Pending |
| SETUP-02 | 22-onboarding-and-session-resilience | Pending |
| SESSION-01 | 22-onboarding-and-session-resilience | Pending |
| SESSION-02 | 22-onboarding-and-session-resilience | Pending |
| PERF-04 | 23-cpu-ttft-and-runtime-watchdog | Pending |
| PERF-05 | 23-cpu-ttft-and-runtime-watchdog | Pending |
| ENT-01 | 24-enterprise-audit-log-mvp | Pending |
| ENT-02 | 25-plugin-sdk-and-ide-bridge-foundation | Pending |
| IDE-01 | 25-plugin-sdk-and-ide-bridge-foundation | Pending |
