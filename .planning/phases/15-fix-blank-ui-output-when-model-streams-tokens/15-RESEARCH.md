# Phase 15 Research: Chat Mode Polish Foundation

**Date:** 2026-03-31
**Phase:** 15 — Chat Mode Polish Foundation
**Scope:** CHAT-01, CHAT-02, CHAT-03, CHAT-04

## Research Objective
Determine the safest implementation path for strict chat-mode sanitization, reasoning marker stripping, and conservative output cleanup without regressing agentic behavior or streaming stability.

## Current Codebase Findings

### Mode & Prompt Routing
- `agent-rs/src/config.rs` loads runtime config from `scripts/config.py` and exposes `exec_mode` via `HELIX_EXEC_MODE`.
- `agent-rs/src/main.rs` currently computes `is_chat_mode` and uses hardcoded prompt branching (`""` for chat, persona prompt for agentic).
- Web path in `agent-rs/src/server.rs` currently calls `crate::expose_think_blocks(content)` unconditionally before emitting text events.

### Reasoning Trace Exposure
- `agent-rs/src/main.rs` exposes `<think>` as `<thinking>` via `expose_think_blocks`.
- `agent-rs/src/tui.rs` includes `<think>` parsing and rendering controls (including visibility toggle), indicating agentic transparency is already deeply integrated.
- Existing behavior from prior phases intentionally surfaced reasoning traces for UX transparency.

### Streaming Path Constraints
- Streaming loops in `main.rs` currently append token chunks immediately and maintain `full_content` for final message materialization.
- TUI receives batched `TokenChunk` events plus heartbeat/status events.
- Any chat filter must be mode-gated and safe for both incremental chunks and finalized output.

### Dependency Baseline
- `agent-rs/Cargo.toml` does not currently include `regex`.
- Existing dependencies include `tokio`, `reqwest`, `serde_json`, `ratatui`, `crossterm`, `futures-util`.
- Adding `regex` is viable but optional; a manual scanner can avoid new dependency surface.

## Decision-Aligned Recommendations

### 1) Prompt Isolation (CHAT-01)
- Add explicit config keys in `scripts/config.py` and bridge fields in `agent-rs/src/config.rs`:
  - `CHAT_SYSTEM_PROMPT`
  - `AGENTIC_SYSTEM_PROMPT`
- In `main.rs`, route prompt by mode from config values (not hardcoded empty chat prompt).
- Keep grammar/tool behavior unchanged in agentic mode.

### 2) Reasoning Marker Removal (CHAT-02)
- Implement `strip_reasoning_blocks(text: &str) -> String` in a shared utility module (`agent-rs/src/utils.rs`).
- Strip block families and their content:
  - `<think>...</think>`
  - `<thinking>...</thinking>`
  - `<analysis>...</analysis>`
- Handle malformed/unclosed tags with fallback scanning logic.
- Apply only when mode is chat in terminal/TUI/web output paths.

### 3) Conservative Cleanup (CHAT-03, CHAT-04)
- Implement pipeline function `clean_chat_output(text: &str) -> String` with ordered stages:
  1. Protect blocks (fenced code, inline code, tool-shaped JSON)
  2. Strip reasoning blocks
  3. Deduplicate consecutive identical sentences
  4. Normalize curly quotes and unambiguous mismatched quotes
  5. Restore protected blocks
- Ensure no global rewrite, no style transformation, no markdown or code mutation.

## Risks & Mitigations

### Risk: Over-filtering user-visible content
- **Mitigation:** Protect-restore strategy for code blocks, inline code, and tool payload JSON.

### Risk: Divergence between terminal and web sanitization paths
- **Mitigation:** Centralize cleanup in shared utilities and call same function from both `main.rs` and `server.rs` in chat mode.

### Risk: Agentic regression
- **Mitigation:** Strict mode gate on all sanitization calls; add regression tests asserting agentic path untouched.

### Risk: Streaming-time partial tags
- **Mitigation:** Apply conservative final-pass cleanup on materialized `full_content`; optional lightweight chunk sanitization should never mutate protected structures.

## Suggested Test Matrix

1. `"hey!"` in chat mode:
- No `thinking` text, no numbered chain-of-thought, concise tone.

2. Reasoning strip:
- `"<think>x</think>Hello"` -> `"Hello"`
- malformed `<think` fragments removed safely.

3. Dedup:
- `"Hello. Hello. How are you?"` -> `"Hello. How are you?"`

4. Quote normalization:
- curly quotes replaced with straight quotes.

5. Preservation:
- fenced code unchanged
- inline code unchanged
- tool-shaped JSON unchanged

6. Agentic non-regression:
- reasoning/transparency behavior remains as before in agentic mode.

## Implementation Notes for Planning

- Preferred file touchpoints:
  - `scripts/config.py`
  - `agent-rs/src/config.rs`
  - `agent-rs/src/main.rs`
  - `agent-rs/src/server.rs`
  - `agent-rs/src/utils.rs` (new)
- Optional `regex` dependency can be introduced if chosen by implementer; manual scanner is acceptable per context discretion.
- Keep Phase 15 scope bounded: do not include Phase 16 byte-level streaming refactor or Phase 17 async tool orchestration changes.

---

**Research conclusion:** Phase 15 is implementable with low architectural risk. The safest approach is shared, mode-gated cleanup utilities plus explicit prompt isolation, with conservative transformations and strong preservation guardrails.

## Rerun Validation (2026-03-31)
- Forced research rerun completed for /gsd-plan-phase 15 --research.
- Re-validated Phase 15 requirement mapping (CHAT-01..CHAT-04) and existing plan split (15-01, 15-02).
- Existing research conclusions remain valid; no new dependency or architecture blocker discovered.
