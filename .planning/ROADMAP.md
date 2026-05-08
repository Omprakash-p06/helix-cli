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

## [ ] Phase 06: Autonomous "Fix It" Mode & Multi-agent Voting
**Goal:** Enable high-trust autonomous operations for routine maintenance and repair.
*   **Tasks:**
    *   Implement user-configurable trust levels (Safe Mode vs. Auto Mode).
    *   Build Guardian multi-agent voting for high-risk action consensus.
    *   Implement blocklist for irreversible/unrecoverable actions.
*   **Success Metrics:** 95% diagnostic accuracy; 0% catastrophic failures in 1000+ test runs.
