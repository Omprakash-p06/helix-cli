---
status: testing
phase: 14-fix-tui-missing-output-bug
source: [14-01-SUMMARY.md]
started: 2026-03-29T16:10:00Z
updated: 2026-03-29T17:08:30Z
---

## Current Test

number: 5
name: Interrupt Streaming and Partial Results
expected: Start a long generation (e.g. "Write a 500 word essay on..."). After a few tokens appear, press Ctrl+C to interrupt. The app should stop gracefully and preserve the partial output that was streamed so far (partial sentence/tokens visible, not blank).
awaiting: user response

## Tests

### 1. Cold Start Smoke Test
**expected:** Kill any running server/service. Clear ephemeral state. Start the application from scratch (agent-rs + Qwen model). Server boots without errors, cargo build completes, and a basic query returns live data without crashing.
**result:** pass

### 2. TUI Streaming Output — Visible Tokens
**expected:** Launch app in TUI mode (`HELIX_UI_MODE=tui`). Type a prompt like "What is 2+2?" and send. Watch the TUI as the model generates tokens. Tokens should appear in real time in the output area (not appear blank). You should see the complete response by the time generation finishes.
**result:** pass

### 3. Terminal Mode Streaming Output
**expected:** Launch app in terminal mode (default, or `HELIX_UI_MODE=terminal`). Type a prompt like "Explain photosynthesis in one sentence" and send. Output should appear streamed to stdout in real time. Generation should not appear blocked or stuck.
**result:** issue
**reported:** "Looks stuck and is taking a lot of time to generate the response"
**severity:** major

### 4. Tool Calls Render While Streaming
**expected:** Send a prompt that may trigger a tool call, e.g. "What files are in the current directory?" or similar. While tool output is being processed, new tokens should still appear. After tool completes, generation should continue. No freezing, no blank output during tool phase.
**result:** issue
**reported:** "no its stuck"
**severity:** major

### 5. Interrupt Streaming and Partial Results
**expected:** Start a long generation (e.g. "Write a 500 word essay on..."). After a few tokens appear, press Ctrl+C to interrupt. The app should stop gracefully and preserve the partial output that was streamed so far (partial sentence/tokens visible, not blank).
**result:** pending

## Summary

total: 5
passed: 2
issues: 2
pending: 1
skipped: 0

## Gaps

- truth: "Terminal mode streaming output appears in real time without blocking"
  status: failed
  reason: "User reported: Looks stuck and is taking a lot of time to generate the response"
  severity: major
  test: 3
  artifacts: [agent-rs/src/main.rs (terminal streaming loop)]
  missing: []

- truth: "Tool calls should render and generation should continue without freezing"
  status: failed
  reason: "User reported: no its stuck"
  severity: major
  test: 4
  artifacts: [agent-rs/src/main.rs (tui loop), agent-rs/src/tui.rs]
  missing: []
