---
phase: 14-fix-tui-missing-output-bug
plan: 01
subsystem: tui-streaming
tags: [rust, sse, tui, streaming, regression]

requires:
  - phase: 13-context-and-discoverability
    provides: Existing TUI streaming event loop and rendering pipeline
provides:
  - Chunk-safe SSE parsing for fragmented frames
  - Restored token streaming visibility in TUI
  - Unit-test coverage for parser edge cases
affects: [streaming-loop, terminal-mode, tui-mode]

tech-stack:
  added: []
  patterns: [buffered-sse-line-parsing, shared-parser-module, parser-unit-tests]

key-files:
  created:
    - agent-rs/src/stream.rs
  modified:
    - agent-rs/src/main.rs

key-decisions:
  - "Parse SSE as line events with a shared parser state instead of raw chunk text.lines()"
  - "Fire GenerationStarted only after first parsed content delta reaches TUI"
  - "Preserve existing tool_call delta accumulation logic while replacing input event source"

patterns-established:
  - "Use stream::SseParser in all bytes_stream loops to avoid partial-line loss"
  - "Keep parser JSON-agnostic (returns data payload strings), decode JSON in orchestrator loops"

requirements-completed: [UX-01, TEST-01]

duration: 20min
completed: 2026-03-29
---

# Phase 14 Plan 01 Summary

Implemented a robust SSE parser and integrated it into both terminal and TUI streaming paths so model output is rendered even when data frames are split across network chunks.

## What was built

- Added `agent-rs/src/stream.rs` with `SseParser` and `SseEvent`.
- Added parser tests for:
  - Fragmented line reconstruction
  - Multiple data lines in a single chunk
  - `[DONE]` handling
  - CRLF and LF newline handling
  - Final partial-line flush on stream end
- Updated `agent-rs/src/main.rs`:
  - Added `mod stream;`
  - Replaced raw `text.lines()` SSE parsing in terminal loop with `SseParser` events
  - Replaced raw `text.lines()` SSE parsing in TUI loop with `SseParser` events
  - Ensured `GenerationStarted` is emitted only after first content token is parsed

## Verification

- `cargo test stream::tests::` passed (5 tests).
- `cargo check -q` passed.

## Outcome

The blank-response symptom in TUI is addressed by eliminating partial-frame loss in SSE parsing. Streaming output and tool-call accumulation both continue to function with parser-backed input.
