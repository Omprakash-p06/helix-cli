# Helix OS Agent Roadmap

## Global Process Standards

### Recording & Reporting
1. **Atomic Commits:** Perform a `git commit` immediately once all tests pass following the completion of a phase.
2. **Quality Enforcement:** Run full code quality tests (`cargo clippy`, `cargo test`) every time a `/gsd-debug` session or a roadmap phase is completed.
3. **Architectural Visualization:** Update `misc/architecture_[YYYY-MM-DD].svg` upon completion of each phase or successful debug fix.

### Preventative Controls & CI/CD
4. **Release Versioning:** Follow **Conventional Commits** (`feat:`, `fix:`, `BREAKING CHANGE:`) to enable automated semantic versioning.
5. **Branching Strategy:** Use **Trunk-Based Development** with mandatory CI gates before merging to `main`.
6. **Quality Gates:** Enforce a minimum **80% test coverage** floor and treat all `clippy` warnings as errors in CI.
7. **Scheduled Validation:** Execute a full nightly CI pipeline to detect regressions or flakiness in the `main` branch.
8. **Supply Chain Security:** Use `cargo deny` or `cargo audit` in CI to block dependencies with known vulnerabilities or unapproved licenses.
9. **Automated Documentation:** Build and deploy `cargo doc` and `mdBook` documentation automatically on every push to `main`.
10. **Artifact Integrity:** Cryptographically sign all binary release artifacts to ensure supply chain security.

This roadmap defines the pivot of Helix Agent into a local-first, autonomous AI OS troubleshooting agent, powered by any local model and orchestrated by GSD 2.0.

## [x] Phase 01: Foundation & Security Sandbox
**Goal:** Establish the core model foundation and the non-negotiable security isolation layer.
**Success Metrics:** All commands execute inside an isolated container; forbidden commands are blocked before execution; full audit trail persists.

## [x] Phase 02: OS Diagnostics & Read-Only Troubleshooting
**Goal:** Enable the agent to safely inspect system state and diagnose issues without making changes.
**Success Metrics:** Agent accurately identifies 80% of common OS issues in a read-only environment.

## [x] Phase 03: Guided Repair & Human-Approved Fixes
**Goal:** Transition from diagnostics to repair with mandatory human-in-the-loop gates.
**Success Metrics:** 0% unexpected system modifications; agent never executes a fix without explicit user approval.

## [x] Phase 04: GSD 2.0 Integration & Autonomous Workflows
**Goal:** Integrate the GSD 2.0 orchestration layer for complex, multi-step repairs.
**Success Metrics:** Agent successfully navigates 5+ step repair workflows with automatic recovery from mid-step failures.

## [x] Phase 05: Model Management & UX Polish
**Goal:** Generalize model support and improve orchestration UX with autofill.
*   **Status:** COMPLETED (4 plans)
*   **Tasks:**
    *   Implement dynamic model discovery in `models/` folder.
    *   Add model selection menu to TUI startup.
    *   Implement Hugging Face GGUF downloader tool.
    *   Add GSD Message autofill (suggest `/gsd plan` / `/gsd execute`).
*   **Plans:** 4 plans (05-01 through 05-04)
*   **Success Metrics:** User can switch models without reconfiguring code; HF models can be downloaded via command; UI suggests the next logical GSD step.

## [x] Phase 06: Autonomous "Fix It" Mode & Multi-agent Voting
**Goal:** Enable high-trust autonomous operations for routine maintenance and repair.
*   **Status:** COMPLETED (3 plans)
*   **Tasks:**
    *   Implement user-configurable trust levels (Safe Mode vs. Auto Mode).
    *   Build Guardian multi-agent voting for high-risk action consensus.
    *   Implement blocklist for irreversible/unrecoverable actions.
*   **Plans:** 3 plans (06-01 through 06-03)
*   **Success Metrics:** 95% diagnostic accuracy; 0% catastrophic failures in 1000+ test runs.

## [x] Phase 07: Security Hardening & Logical Vulnerability Remediation
**Goal:** Close the P0 and P1 logical security gaps identified in the vulnerability audit — make ToolRuntime the single execution gateway, enforce absolute capability boundaries, harden interpreter/build-tool execution paths, and add adversarial regression tests.
*   **Status:** GAP CLOSURE (2 plans)
*   **Plans:** 2 plans (07-01, 07-02)
*   **Success Metrics:** Zero bypass paths exist from any tool to unsandboxed interpreter/build execution; ReadOnly is an absolute incapability (no write/exec bypass); all security guardrail tests pass including adversarial path-traversal and command-injection scenarios; new `capabilities` module replaces persona-name-based policy checks. Provenance-based content filtering prevents Untrusted content from reaching system prompts. Watchdog verifies process ownership before terminating (P1-4).

## [ ] Phase 08: Repository Map & Context Engineering Layer
**Goal:** Build a hierarchical context system — Tree-sitter/LSP symbol extraction, dependency graph, just-in-time retrieval, and a durable agent memory layer — so the agent can reason over large codebases without concatenating raw files into the prompt.
*   **Status:** COMPLETED (5 plans)
*   **Plans:** 5 plans (08-01 through 08-05)
    - [x] 08-01 — Context Module Foundation: Types, Token Budget & Skeleton Extraction
    - [x] 08-02 — Tree-sitter Symbol Indexer & SQLite Cache
    - [x] 08-03 — Durable Agent Memory Layer (SQLite FTS5)
    - [x] 08-04 — JIT Retrieval Pipeline, Context Engine Integration & search_codebase Tool
    - [x] 08-05 — Populate Import Edges Table & Dependency Graph (gap closure)
*   **Success Metrics:** Agent can locate any symbol, all callers, and direct dependencies within 3 seconds; context budget stays under 40k active tokens per task; durable state (goals, constraints, decisions, failed attempts, edit ledger) survives compaction cycles without information loss.

## [ ] Phase 09: Web Research Agent — Deep Research Pipeline
**Goal:** Add a bounded web research subsystem (Planner → Source Workers → Evidence Store → Synthesizer) that gathers cited external intelligence before coding changes, with strict prompt-injection isolation.
**Success Metrics:** Research completes with provenance-stamped citations; fetched content is never treated as executable instructions; freshness classifier correctly routes stale dependency questions to live research; research brief is ≤2k tokens delivered to the coding agent.

## [ ] Phase 10: Gemma 4 E4B Integration & Evaluation Suite
**Goal:** Switch the default test model to Gemma 4 E4B-it (128K context, native function calling), wire it into the updated context-engineering stack, and run the minimum evaluation suite to benchmark repo comprehension, tool-call correctness, long-session retention, research factuality, prompt-injection resistance, policy-escape resistance, and rollback correctness.
**Success Metrics:** All 8 evaluation scenarios pass; long-session retention benchmark shows ≥90% constraint recall after 3 compaction cycles; prompt-injection tests show 0% instruction-following from untrusted content.
