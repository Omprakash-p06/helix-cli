# Phase 17: Non-Blocking Tool Execution - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-06
**Phase:** 17-non-blocking-tool-execution
**Areas discussed:** Status Display, Timeout Handling, Parallel Failure Strategy, Result Ordering

---

## Status Display

| Option | Description | Selected |
|--------|-------------|----------|
| Inline in chat area (like Claude Code) | Works in both terminal and TUI modes. Users see progress naturally within conversation flow. | ✓ |
| Status bar (TUI only) | Less discoverable and can be ignored. |

**User's choice:** Inline in chat area (like Claude Code)
**Notes:** Works in both terminal and TUI modes, provides better UX than status bar only.

---

## Timeout Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Return timeout error to LLM (let LLM decide retry/fallback) | Gives the LLM agency to handle errors intelligently (e.g., try a different approach, ask user, or fall back). Auto-retry may repeat the same failure and waste time. Prompting user breaks automation. | ✓ |
| Auto-retry once | Quick recovery but may repeat the same failure |
| Prompt user | Breaks automation, not acceptable |

**User's choice:** Return timeout error to LLM (let LLM decide retry/fallback)
**Notes:** Best for AI agents - gives LLM agency to handle errors intelligently.

---

## Parallel Failure Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Report all results to LLM (including failures) – let critic decide | Provides full information so the LLM can make an informed decision (e.g., proceed with partial results, retry failed tools, or ask for help). Stopping at first failure may discard successful parallel work. | ✓ |
| Stop at first failure | Fast fail but may lose successful parallel work |

**User's choice:** Report all results to LLM (including failures) – let critic decide
**Notes:** Provides full information for informed LLM decision-making.

---

## Result Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| Original call order (preserves LLM expectations) | LLMs expect tool results in the same order they were called; mixing order can confuse reasoning and break dependencies. Correctness > speed for agent reliability. | ✓ |
| Completion order (fastest first) | May confuse LLM reasoning if tool order matters |

**User's choice:** Original call order (preserves LLM expectations)
**Notes:** LLMs expect tool results in same order called - correctness > speed.

---

## the agent's Discretion

- TUI event timing (immediate vs debounced) — standard approach acceptable
- Specific logging for tool lifecycle events — standard debug logging acceptable

---

## Deferred Ideas

None — discussion stayed within phase scope.