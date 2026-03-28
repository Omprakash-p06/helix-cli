---
phase: 10
slug: terminal-ui-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | none — using standard cargo |
| **Quick run command** | `cargo check` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | REQ-01 | unit | - | ❌ W0 | ⚠️ manual |
| 10-01-02 | 01 | 1 | REQ-02 | smoke | - | ❌ W0 | ⚠️ manual |
| 10-01-03 | 01 | 1 | REQ-03 | smoke | - | ❌ W0 | ⚠️ manual |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky · ⚠️ manual*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ratatui foundation | REQ-01 | Terminal backend requires real terminal or PTY simulation not set up. | Run `cargo run`. Ensure welcome banner displays. |
| Input Layer & Autocomplete | REQ-02 | Internal UI logic is private and not exposed for unit tests. | Type `/he`, expect phantom `lp`. |
| Multiline & Command Preview | REQ-03 | Internal UI logic is private. | Press `Alt+Enter` to see the preview. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
