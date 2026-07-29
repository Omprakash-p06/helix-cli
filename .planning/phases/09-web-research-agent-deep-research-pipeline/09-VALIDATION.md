---
phase: 09
slug: web-research-agent-deep-research-pipeline
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-29
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) / pytest 9.0 (Python) |
| **Config file** | `agent-rs/Cargo.toml` / `pyproject.toml` |
| **Quick run command** | `cargo test --package agent-rs web_research -q` |
| **Full suite command** | `cargo test --package agent-rs -q && pytest tests/` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --package agent-rs web_research -q`
- **After every plan wave:** Run `cargo test --package agent-rs -q`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-T1 | 01 | 1 | Cargo Dependencies | compile | `cargo check --package agent-rs -q` | ✅ | ✅ green |
| 09-01-T2 | 01 | 1 | Module Skeleton | compile | `cargo check --package agent-rs -q` | ✅ | ✅ green |
| 09-01-T3 | 01 | 1 | Freshness Classifier | unit | `cargo test --package agent-rs web_research::classifier -q` | ✅ | ✅ green |
| 09-01-T4 | 01 | 1 | SQLite Schema | unit | `cargo test --package agent-rs web_research::store -q` | ✅ | ✅ green |
| 09-01-T5 | 01 | 1 | EvidenceStore CRUD | unit | `cargo test --package agent-rs web_research::store -q` | ✅ | ✅ green |
| 09-02-T1 | 02 | 1 | Sanitization Pipeline | unit | `cargo test --package agent-rs web_research::sanitize -q` | ✅ | ✅ green |
| 09-02-T2 | 02 | 1 | Sanitizer Security | unit | `cargo test --package agent-rs web_research::sanitize -q` | ✅ | ✅ green |
| 09-03-T1 | 03 | 2 | ResearchPlanner | unit | `cargo test --package agent-rs web_research::planner -q` | ✅ | ✅ green |
| 09-03-T2 | 03 | 2 | WorkerPool | integration | `cargo test --package agent-rs web_research::worker -q` | ✅ | ✅ green |
| 09-03-T3 | 03 | 2 | WebResearchPipeline | integration | `cargo test --package agent-rs web_research -q` | ✅ | ✅ green |
| 09-04-T1 | 04 | 3 | EvidenceSynthesizer | unit | `cargo test --package agent-rs web_research::synthesizer -q` | ✅ | ✅ green |
| 09-04-T2 | 04 | 3 | Context Integration | integration | `cargo test --package agent-rs context -q` | ✅ | ✅ green |
| 09-04-T3 | 04 | 3 | Adversarial Isolation | integration | `cargo test --package agent-rs --test web_research_adversarial -q` | ✅ | ✅ green |
| 09-04-T4 | 04 | 3 | Module Re-exports | integration | `cargo test --package agent-rs -q` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-07-30

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-30
