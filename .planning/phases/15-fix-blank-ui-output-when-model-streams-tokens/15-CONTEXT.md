# Phase 15: fix-blank-ui-output-when-model-streams-tokens - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement chat-mode polish foundations: strict chat prompt isolation, reasoning-trace stripping, and conservative output cleanup so chat responses are direct and professional while leaving agentic-mode transparency behavior unchanged.

</domain>

<decisions>
## Implementation Decisions

### Mode Boundary
- **D-01:** Apply chat-only sanitization in both terminal and TUI paths when execution mode is chat.
- **D-02:** Agentic mode remains behaviorally unchanged for reasoning visibility/transparency.
- **D-03:** Chat mode must never display thought traces to users.

### Reasoning Marker Policy
- **D-04:** Strip reasoning blocks and their content for `<think>...</think>`, `<thinking>...</thinking>`, and `<analysis>...</analysis>`.
- **D-05:** Remove malformed/unclosed reasoning tags using fallback heuristics when strict block matching is not possible.
- **D-06:** Preferred matching approach is non-greedy pattern removal per marker family; malformed cleanup may be line-scanned.

### Output Cleanup Strictness
- **D-07:** Use conservative cleanup only (no stylistic rewriting).
- **D-08:** Deduplicate only consecutive identical sentences (exact match, case-sensitive, trailing-space-insensitive).
- **D-09:** Normalize curly quotes to straight quotes.
- **D-10:** Fix mismatched quotes only when unambiguous.
- **D-11:** Do not alter markdown links, inline code, or fenced code content.

### Content Preservation Rules
- **D-12:** Never alter fenced code blocks.
- **D-13:** Never alter inline code spans.
- **D-14:** Never alter tool JSON payload blocks (tool-call shaped JSON).
- **D-15:** Processing order is mandatory: identify/protect blocks first, then reasoning stripping and cleanup over remaining prose.

### Phase Plan Split
- **D-16:** Keep two plan files: 15-01 (prompt + mode + reasoning filter) and 15-02 (dedupe + quote normalization + protected-block preservation).

### the agent's Discretion
- Exact fallback implementation details for malformed tag handling (regex-first vs scanner-first), as long as D-04/D-05 are preserved.
- Whether streaming filtering is chunk-local or end-accumulation with conservative behavior, as long as chat mode never leaks reasoning traces.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone Scope and Requirements
- `.planning/ROADMAP.md` — Phase 15 scope, dependencies, and plan split anchor.
- `.planning/REQUIREMENTS.md` — CHAT-01..CHAT-04 requirement definitions for this phase.
- `.planning/PROJECT.md` — milestone intent and active constraints for v1.2.
- `.planning/STATE.md` — current execution position and roadmap context.

### Prior Phase Decisions Affecting Phase 15
- `.planning/phases/07-generation-speed-thought-ui/07-CONTEXT.md` — prior decision to expose think tags; Phase 15 introduces chat-mode-only suppression.
- `.planning/phases/11-output-polish-and-streaming/11-CONTEXT.md` — streaming and think rendering expectations in TUI.
- `.planning/phases/12-control-and-feedback/12-CONTEXT.md` — interaction/control behaviors to preserve while adding cleanup.
- `.planning/phases/14-fix-tui-missing-output-bug/14-02-SUMMARY.md` — current streaming reliability constraints and heartbeat behavior.

### Research Inputs for v1.2
- `.planning/research/SUMMARY.md` — synthesized findings and pitfalls for chat filtering.
- `.planning/research/FEATURES.md` — cleanup and filtering patterns.
- `.planning/research/ARCHITECTURE.md` — integration sequence and file touchpoints.
- `.planning/research/PITFALLS.md` — edge cases to avoid (over-filtering, UTF-8/stream behavior).

### Implementation Touchpoints
- `agent-rs/src/main.rs` — mode routing, prompt injection, streaming path integration.
- `agent-rs/src/tui.rs` — rendered output path for chat-mode sanitization behavior.
- `agent-rs/src/server.rs` — web/SSE output transformation path.
- `agent-rs/src/types.rs` — mode/message typing constraints.
- `agent-rs/src/tools.rs` — tool payload shape used for protected JSON detection.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `expose_think_blocks` in `agent-rs/src/main.rs` already centralizes reasoning-tag exposure behavior and is the primary hook to split chat vs agentic behavior.
- `extract_visible_delta_text` and stream token buffering in `agent-rs/src/main.rs` provide insertion points for conservative chat cleanup.
- `TuiEvent::TokenChunk`, `ResponseDone`, and rendering flow in `agent-rs/src/tui.rs` enable chat-mode-safe display enforcement.
- Web streaming path in `agent-rs/src/server.rs` already transforms outgoing text and can mirror chat filtering.

### Established Patterns
- Mode decisions currently rely on `app_config.exec_mode` and environment-driven UI selection (`HELIX_UI_MODE`).
- Streaming currently uses SSE parsing + timed flush in terminal/TUI loops.
- Tool calls are assembled from incremental deltas and executed in orchestrator loops; payload shape is consistent and detectable.

### Integration Points
- Chat/agentic boundary enforcement at request construction and post-response cleanup in `agent-rs/src/main.rs`.
- UI-safe rendering parity in `agent-rs/src/tui.rs` and `agent-rs/src/server.rs` so chat mode is consistently sanitized across terminal/TUI/web paths.
- Utility extraction (`utils.rs`) for shared cleanup pipeline invoked by both full-response and streaming flows.

</code_context>

<specifics>
## Specific Ideas

- Add prompt keys for `chat_system` and `agentic_system` and route by mode.
- Implement `strip_reasoning_blocks(text: &str) -> String`, `deduplicate_consecutive_sentences(text: &str) -> String`, `normalize_quotes(text: &str) -> String`, and `clean_chat_output(text: &str) -> String`.
- Protect/restore blocks workflow for fenced code, inline code, and tool-shaped JSON payloads before prose cleanup.
- Maintain two sub-plans with acceptance tests exactly as specified in the decision pack.

</specifics>

<deferred>
## Deferred Ideas

- Broader streaming refactor (byte-level no-buffer policy) remains Phase 16 scope.
- Non-blocking/parallel tool execution remains Phase 17 scope.
- Shared types crate/tracing/clippy-hardening remains Phase 18 scope.

None — discussion stayed within Phase 15 boundary for current decisions.

</deferred>

---

*Phase: 15-fix-blank-ui-output-when-model-streams-tokens*
*Context gathered: 2026-03-29*
