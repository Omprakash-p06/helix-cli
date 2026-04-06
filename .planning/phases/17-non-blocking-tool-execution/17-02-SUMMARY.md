---
plan: 17-02
phase: 17-non-blocking-tool-execution
status: complete
completed: "2026-04-06T18:05:00Z"
---

# Plan 17-02: Parallel Execution Correctness

## Objective
Ensure parallel execution correctness with proper result ordering and timeout handling.

## What Was Built
- **Result ordering**: Index tracking added to `join_all` tasks, results sorted by original call index before processing (D-04)
- **Timeout error handling**: Verified 30s timeout returns clear error message to LLM (no auto-retry)
- **Parallel failure strategy**: All results pushed to messages regardless of success/failure (D-03)

## Key Files Modified
- `agent-rs/src/main.rs`: Index tracking and sort_by_key in both TUI and terminal mode tool execution blocks

## Commits
1. `feat(17-02): ensure result ordering and parallel execution correctness`

## Requirements Met
- TOOL-04: Results maintain original call order via `sort_by_key(|(idx, _)| *idx)`
- TOOL-05: Timeout produces clear error message for LLM
- D-03: All results reported including failures

## Self-Check: PASSED
- `sort_by_key` present in both TUI (line 1891) and terminal (line 1369) modes
- Timeout error message: "Tool '{}' timed out after 30 seconds"
- All tool results pushed to messages (not filtered by success)
