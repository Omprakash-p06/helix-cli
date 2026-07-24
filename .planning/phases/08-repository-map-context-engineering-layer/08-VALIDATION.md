---
phase: 8
slug: repository-map-context-engineering-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-25
---

# Phase 08 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (unit + integration) |
| **Config file** | `agent-rs/Cargo.toml` |
| **Quick run command** | `cargo test context:: -p agent-rs -- --nocapture` |
| **Full suite command** | `cargo test -p agent-rs -- --nocapture && cargo clippy -p agent-rs -- -D warnings` |
| **Estimated runtime** | ~30 seconds (unit) / ~60 seconds (full with clippy) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test context:: -p agent-rs`
- **After every plan wave:** Run `cargo test -p agent-rs && cargo clippy -p agent-rs -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 08-??-01 | TBD | 0 | Wave 0 | unit stub | `cargo test context::` | ❌ W0 | ⬜ pending |
| 08-??-02 | TBD | 1 | Symbol extraction | unit | `cargo test context::indexer` | ❌ W0 | ⬜ pending |
| 08-??-03 | TBD | 1 | Dependency graph | unit | `cargo test context::graph` | ❌ W0 | ⬜ pending |
| 08-??-04 | TBD | 2 | JIT retrieval | integration | `cargo test context::retrieval` | ❌ W0 | ⬜ pending |
| 08-??-05 | TBD | 2 | Token budget | unit | `cargo test context::budget` | ❌ W0 | ⬜ pending |
| 08-??-06 | TBD | 3 | Memory persistence | integration | `cargo test context::memory` | ❌ W0 | ⬜ pending |
| 08-??-07 | TBD | 3 | Benchmark <3s | bench | `cargo test --release -- benchmark` | ❌ W0 | ⬜ pending |
| 08-??-08 | TBD | 3 | Compaction survival | integration | `cargo test context::memory::test_session_survives` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `agent-rs/src/context/mod.rs` — module stub with public API signatures
- [ ] `agent-rs/src/context/indexer.rs` — Tree-sitter indexer stub
- [ ] `agent-rs/src/context/retrieval.rs` — retrieval pipeline stub
- [ ] `agent-rs/src/context/memory.rs` — SQLite memory layer stub
- [ ] `agent-rs/src/context/skeleton.rs` — distillation stub
- [ ] `agent-rs/src/context/budget.rs` — token budget stub
- [ ] `agent-rs/tests/context_integration.rs` — integration test stubs for compaction + budget tests
- [ ] Remove broken `use fastembed::...` import from `agent-rs/src/rag.rs` before adding new deps

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Repo skeleton injected at session start | Context pre-injection | Requires live LLM session | Start helix-agent, observe context prefix in first LLM call |
| `search_codebase` tool called by LLM | Tool invocation | Requires live LLM | Ask agent about `ToolRuntime`, verify it calls search_codebase |
| DOT graph export correctness | Architecture visualization | Visual inspection | Run `--export-graph`, open misc/architecture_*.svg |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
