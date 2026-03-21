# Roadmap: Helix Agent

## Overview

This roadmap focuses on making Helix reliably fast, local-first, and low-friction for agentic use on low-end laptops. The sequence first stabilizes startup/runtime, then hardens low-end performance defaults, then strengthens tool-calling reliability, and finally polishes chat UX to feel like a normal local chatbot.

## Phases

- [ ] **Phase 1: Runtime Stability** - Make startup and backend orchestration resilient and diagnosable.
- [ ] **Phase 2: Low-End Optimization** - Tune defaults and setup behavior for constrained laptops.
- [ ] **Phase 3: Tool-Calling Reliability** - Ensure agentic tools are consistent across local backends and OSes.
- [ ] **Phase 4: Chat UX Normalization** - Remove orchestration noise and make interaction natural.

## Phase Details

### Phase 1: Runtime Stability
**Goal**: Local server/orchestrator startup is robust, understandable, and model-selectable.
**Depends on**: Nothing (first phase)
**Requirements**: [RT-01, RT-02, RT-03, RT-04]
**Success Criteria** (what must be TRUE):
  1. User can launch Helix and reach a ready prompt with one command.
  2. Startup failures include clear reason plus log file paths.
  3. Backend fallback works automatically when primary backend fails.
**Plans**: TBD

Plans:
- [ ] 01-01-PLAN.md - Startup path hardening and logging contracts
- [ ] 01-02-PLAN.md - Backend detection/fallback and model selection reliability

### Phase 2: Low-End Optimization
**Goal**: Defaults are safe and fast on low-end systems without heavy setup overhead.
**Depends on**: Phase 1
**Requirements**: [PERF-01, PERF-02, PERF-03]
**Success Criteria** (what must be TRUE):
  1. Setup completes without forcing expensive checks by default.
  2. Low-end defaults avoid common OOM/timeout startup failures.
  3. Users can adjust context/perf profiles via config or env, no code edits.
**Plans**: TBD

Plans:
- [ ] 02-01-PLAN.md - Setup and preflight policy simplification
- [ ] 02-02-PLAN.md - Adaptive runtime defaults for constrained hardware

### Phase 3: Tool-Calling Reliability
**Goal**: Agent tool execution is stable across Windows/POSIX and local backend variants.
**Depends on**: Phase 2
**Requirements**: [TOOL-01, TOOL-02, TOOL-03]
**Success Criteria** (what must be TRUE):
  1. Core file tools succeed consistently within sandbox boundaries.
  2. Terminal tool behaves correctly on Windows and Linux/macOS.
  3. Tool-call argument compatibility no longer breaks local endpoint loops.
**Plans**: TBD

Plans:
- [ ] 03-01-PLAN.md - Tool schema/execution contract hardening
- [ ] 03-02-PLAN.md - Endpoint compatibility and regression checks

### Phase 4: Chat UX Normalization
**Goal**: Interactive experience feels like a clean local chatbot for daily use.
**Depends on**: Phase 3
**Requirements**: [UX-01, UX-02, UX-03]
**Success Criteria** (what must be TRUE):
  1. Internal reasoning/debug chatter is hidden in normal chat.
  2. Users can chat immediately after launch without mode friction.
  3. CLI output is concise and user-facing by default.
**Plans**: TBD

Plans:
- [ ] 04-01-PLAN.md - Interactive output cleanup and behavior toggles

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Runtime Stability | 0/2 | Not started | - |
| 2. Low-End Optimization | 0/2 | Not started | - |
| 3. Tool-Calling Reliability | 0/2 | Not started | - |
| 4. Chat UX Normalization | 0/1 | Not started | - |
