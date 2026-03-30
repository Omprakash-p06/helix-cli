---
phase: 14-fix-tui-missing-output-bug
plan: 02
subsystem: streaming-visibility
tags: [rust, tui, streaming, gap-closure, uat]

requires:
  - phase: 14-fix-tui-missing-output-bug
    provides: Baseline SSE parser integration from 14-01
provides:
  - Visible streaming text extraction beyond content-only deltas
  - TUI heartbeat rendering during non-token/tool-only streaming windows
  - Interrupt-safe token buffer flushing and regression tests
affects: [streaming-loop, terminal-mode, tui-mode, interrupt-path]

key-files:
  created: []
  modified:
    - agent-rs/src/main.rs
    - agent-rs/src/tui.rs

requirements-completed: [UX-01, TEST-01]

completed: 2026-03-29
---

# Phase 14 Plan 02 Summary

Closed Phase 14 UAT gaps where generation appeared stuck or blank by improving visible token extraction, adding in-chat streaming heartbeat feedback, and guaranteeing partial token flush on interrupt.

## What Was Built

- Updated `agent-rs/src/main.rs`:
  - Added `extract_visible_delta_text()` to read user-visible text from multiple delta keys (`content`, `reasoning_content`, `text`, `response`) instead of relying only on `delta.content`.
  - Reused the helper in both terminal and TUI stream loops.
  - Added `flush_token_buffer()` helper and used it for:
    - periodic token flushes,
    - stream end flush,
    - interrupt/quit branch flush before break.
  - Added heartbeat emission (`TuiEvent::StreamingHeartbeat`) while stream is active but no token chunk is currently flushable.
  - Added regression tests covering non-content visible text extraction and token-buffer flush behavior.

- Updated `agent-rs/src/tui.rs`:
  - Added `TuiEvent::StreamingHeartbeat(String)` handling.
  - Added `streaming_heartbeat` state to `TuiApp`.
  - Rendered heartbeat text in the chat panel when generation is active but no visible token chunk is yet present.
  - Cleared heartbeat on token append and response completion.

## Verification

- `cd agent-rs && cargo check -q` passed.
- `cd agent-rs && cargo test -q` passed.

## Outcome

The UI now provides continuous visible progress during generation, including tool-only/non-content streaming windows, and interrupting generation preserves already-buffered partial output instead of dropping it.
