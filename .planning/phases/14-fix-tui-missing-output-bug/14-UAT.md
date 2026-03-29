---
status: diagnosed
phase: 14-fix-tui-missing-output-bug
source: [14-01-SUMMARY.md]
started: 2026-03-29T16:10:00Z
updated: 2026-03-29T17:16:30Z
---

## Current Test

[testing complete]

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
**result:** issue
**reported:** "live token generation is not visible"
**severity:** major

## Summary

total: 5
passed: 2
issues: 3
pending: 0
skipped: 0

## Gaps

- truth: "Terminal mode streaming output appears in real time without blocking"
  status: failed
  reason: "User reported: Looks stuck and is taking a lot of time to generate the response"
  severity: major
  test: 3
  root_cause: "Streaming loop only renders `delta.content`; when backend emits long `reasoning_content`/non-content deltas first, user sees no live text and perceives a stall."
  artifacts:
    - path: "agent-rs/src/main.rs"
      issue: "Terminal streaming branch ignores non-`content` text deltas and provides no interim progress text."
  missing:
    - "Handle additional streaming text keys (e.g., reasoning/text deltas) as renderable token chunks."
    - "Emit explicit progress/status ticks when receiving non-renderable deltas to avoid stuck perception."

- truth: "Tool calls should render and generation should continue without freezing"
  status: failed
  reason: "User reported: no its stuck"
  severity: major
  test: 4
  root_cause: "TUI stream path emits visible output only for `delta.content`; tool-call assembly and other delta types do not produce interim UI feedback, so turns can look frozen until completion."
  artifacts:
    - path: "agent-rs/src/main.rs"
      issue: "TUI streaming branch accumulates `tool_calls` silently and delays user-visible feedback."
    - path: "agent-rs/src/tui.rs"
      issue: "No dedicated event/render path for tool-call delta heartbeat."
  missing:
    - "Emit lightweight TUI events while tool-call deltas stream (e.g., assembling tool call, receiving args)."
    - "Show non-content streaming indicator in chat area, not only status bar, during tool-only phases."

- truth: "Live token generation should be visible before completion"
  status: failed
  reason: "User reported: live token generation is not visible"
  severity: major
  test: 5
  root_cause: "On interrupt path, buffered tokens can be dropped because the loop breaks immediately without flushing token_buffer; additionally, visibility relies on content deltas only, hiding other streamed text forms."
  artifacts:
    - path: "agent-rs/src/main.rs"
      issue: "Interrupt branch breaks before flushing buffered tokens to TokenChunk."
    - path: "agent-rs/src/main.rs"
      issue: "Generation-start and render pipeline are gated to content deltas only."
  missing:
    - "Flush token_buffer before exiting on interrupt/quit so partial output is preserved."
    - "Broaden visible streaming sources beyond delta.content (reasoning/text/tool-progress signals)."
