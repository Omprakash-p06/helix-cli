# Phase 15: fix-blank-ui-output-when-model-streams-tokens - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 15-fix-blank-ui-output-when-model-streams-tokens
**Areas discussed:** Mode Boundary, Reasoning Marker Policy, Output Cleanup Strictness, Content Preservation Rules

---

## Mode Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Chat sanitization in terminal only | Enforce sanitization only in terminal path | |
| Chat sanitization in terminal + TUI | Enforce chat sanitization in all local UI outputs | ✓ |
| Apply sanitization in both chat and agentic | Simplifies output behavior but removes agentic transparency | |

**User's choice:** Apply chat-only sanitization in terminal and TUI; keep agentic mode unchanged.
**Notes:** Chat mode must never show thought traces.

---

## Reasoning Marker Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Strip `<think>` only | Minimal marker handling | |
| Strip `<think>`, `<thinking>`, `<analysis>` blocks + malformed tag fallback | Broad coverage with safe fallback behavior | ✓ |
| Preserve tags but hide in UI | Keep tags in content, suppress at render-time only | |

**User's choice:** Strip marker families and their content completely, including malformed/unclosed tags with fallback heuristics.
**Notes:** Non-greedy matching pattern preferred; malformed scanner fallback allowed.

---

## Output Cleanup Strictness

| Option | Description | Selected |
|--------|-------------|----------|
| Aggressive rewrite | Rephrase for fluency and remove repeated motifs globally | |
| Conservative artifact-only cleanup | Exact consecutive sentence dedupe + quote normalization only | ✓ |
| No cleanup | Only remove reasoning markers, preserve all other artifacts | |

**User's choice:** Conservative cleanup only.
**Notes:** Dedup is exact/case-sensitive; quote fixes should be minimal and unambiguous.

---

## Content Preservation Rules

| Option | Description | Selected |
|--------|-------------|----------|
| Clean everything uniformly | Simpler pipeline, higher corruption risk | |
| Protect code/tool structures first, clean prose only | Preserves semantics while cleaning user-visible text | ✓ |
| Preserve fenced code only | Protects block code but risks inline/tool payload mutation | |

**User's choice:** Preserve fenced code, inline code, and tool JSON payload blocks before cleanup.
**Notes:** Mandatory processing order: protect -> strip reasoning -> cleanup -> restore.

---

## the agent's Discretion

- Implementation shape for malformed-tag fallback (regex-first vs scanner-first).
- Streaming-time filtering strategy (chunk-safe vs buffered-final), constrained by no chat-mode leak.

## Deferred Ideas

- Byte-level streaming/render refactor is deferred to Phase 16.
- Non-blocking parallel tool execution is deferred to Phase 17.
- Shared types/tracing/clippy-hardening is deferred to Phase 18.
