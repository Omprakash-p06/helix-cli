---
status: resolved
trigger: "In agentic mode, Helix does not call local tools and answers as if it has no filesystem access; response formatting is also poor/clumped."
created: 2026-03-30T10:15:10+05:30
updated: 2026-03-30T12:30:30+05:30
---

## Current Focus
<!-- OVERWRITE on each update - reflects NOW -->

hypothesis: verified fixed by user in real TUI workflow
test: human verification checkpoint response
expecting: no further issue reports for this symptom chain
next_action: archive session, update knowledge base, and finalize commits

## Symptoms
<!-- Written during gathering, then IMMUTABLE -->

expected: in agentic mode, Helix should call local tools (for example list_directory) for filesystem questions and present cleanly formatted assistant output
actual: Helix replies with generic text that it cannot access local filesystem and asks user to run terminal commands manually; output formatting appears clumped and poorly structured
errors: no explicit runtime stack trace shown in this report
reproduction: run TUI in agentic mode and ask a local-files question such as "list out the files that are there in my directory"; observe no tool call and a poorly formatted generic response
started: reported as a new issue now in current session

## Eliminated
<!-- APPEND only - prevents re-investigating -->

## Evidence
<!-- APPEND only - facts discovered -->

- timestamp: 2026-03-30T10:16:20+05:30
	checked: .planning/debug/knowledge-base.md
	found: no known-pattern entry overlaps this symptom set (agentic mode claims + no tool calling + formatting complaints)
	implication: continue fresh root-cause investigation

- timestamp: 2026-03-30T10:17:10+05:30
	checked: agent-rs/src/config.rs and agent-rs/src/main.rs mode initialization
	found: exec_mode is sourced from HELIX_EXEC_MODE with default "chat"; main computes is_chat_mode from exec_mode and sets tools_payload to [] in chat mode
	implication: unless HELIX_EXEC_MODE is explicitly set to agentic, no tools are available at all

- timestamp: 2026-03-30T10:17:50+05:30
	checked: TUI system-command handling in agent-rs/src/main.rs and slash-command list in agent-rs/src/tui.rs
	found: /mode appears in autocomplete but only /clear is handled in TUI command handler; other slash commands are treated as unknown
	implication: users can believe mode switching exists in TUI while runtime mode remains unchanged

- timestamp: 2026-03-30T10:20:30+05:30
	checked: prompt-mode A/B run with same filesystem question
	found: default mode reproduced generic no-access response exactly; forcing HELIX_EXEC_MODE=agentic changed runtime path (grammar compilation + no generic disclaimer response)
	implication: no-tool behavior aligns with chat-mode execution path, confirming mode-selection as primary cause

- timestamp: 2026-03-30T10:21:20+05:30
	checked: agent-rs/src/tui.rs assistant rendering logic
	found: assistant entries with spans are rendered as a single Line regardless of embedded newline characters
	implication: markdown/list/code formatting can appear clumped in TUI even when model output contains line breaks

- timestamp: 2026-03-30T10:26:45+05:30
	checked: code changes in agent-rs/src/main.rs and agent-rs/src/tui.rs
	found: added runtime `/mode` handling (status/switch), synchronized system-prompt updates per mode, fixed `/clear` behavior for no-system chat mode, and changed assistant rendering to preserve multiline spans/content
	implication: mode-selection ambiguity is removed and TUI output should retain intended formatting

- timestamp: 2026-03-30T10:28:40+05:30
	checked: cargo fmt and cargo test after final patches
	found: formatting completed and test suite passed (21/21)
	implication: changes are stable at compile/test level

- timestamp: 2026-03-30T10:28:55+05:30
	checked: prompt-mode A/B run after fixes
	found: default mode still reproduces generic no-tool response; agentic mode follows separate runtime path with tool grammar enabled
	implication: mode boundary is explicitly enforced; interactive TUI behavior now needs user confirmation for end-to-end UX

- timestamp: 2026-03-30T10:36:20+05:30
	checked: user checkpoint response and screenshot in agentic mode
	found: mode banner shows agentic active, user prompts are accepted into history, but no assistant output or tool-call events appear
	implication: issue persists beyond mode-selection UX; response visibility path for agentic likely suppresses fallback/diagnostic output

- timestamp: 2026-03-30T10:39:30+05:30
	checked: patched stream parsing and replay visibility + automated verification
	found: tests pass (24/24) and agentic prompt now shows explicit fallback text instead of blank UI, but still produces no tool calls/content for list-directory query
	implication: silent-output bug is fixed; remaining root cause is upstream generation behavior under current agentic grammar/tool request configuration

- timestamp: 2026-03-30T10:41:10+05:30
	checked: backend-aware grammar policy change + runtime reproductions
	found: with grammar disabled for non-kobold backend, agentic list-directory prompt invoked list_directory tool and produced visible answer; agentic greeting prompt produced normal visible text
	implication: agentic no-output/no-calling behavior was driven by grammar incompatibility on this backend and is now resolved in automated verification

- timestamp: 2026-03-30T10:45:11+05:30
	checked: user checkpoint response including panic trace + agent-rs/src/tui.rs tool result rendering branch
	found: panic points to byte-index slicing in `format!("{}...", &result.output[..200])`; `result.output` contains UTF-8 tree characters like `│` from list_directory output
	implication: crash is deterministic UTF-8 boundary violation during tool preview truncation; fix is localized to TUI rendering helper

- timestamp: 2026-03-30T10:46:30+05:30
	checked: agent-rs/src/tui.rs truncation path
	found: replaced byte-slice truncation with char-boundary-safe helper (`truncate_preview_preserving_utf8`) and added regression tests for Unicode + ASCII truncation
	implication: tool-result previews should no longer panic on box-drawing Unicode output

- timestamp: 2026-03-30T10:46:55+05:30
	checked: cargo fmt and cargo test after truncation patch
	found: verification succeeded with 27/27 tests passing, including new tui truncation tests
	implication: UTF-8 truncation fix is stable in automated verification; pending real TUI confirmation

- timestamp: 2026-03-30T10:50:20+05:30
	checked: TUI-path smoke run (`HELIX_UI_MODE=tui HELIX_EXEC_MODE=agentic --prompt "list out the files..."`)
	found: agentic list_directory flow rendered visible output in TUI without panic on Unicode directory tree content
	implication: previously crashing code path is now stable end-to-end in automated TUI execution

- timestamp: 2026-03-30T12:12:10+05:30
	checked: user checkpoint confirmation and follow-up UX report
	found: user confirmed panic/tool-output fix works but reported inability to scroll down to view the next response after a second prompt
	implication: core agentic/tool path is fixed; remaining issue is isolated to TUI navigation behavior

- timestamp: 2026-03-30T12:17:26+05:30
	checked: agent-rs/src/tui.rs key handling and draw-time scroll computation
	found: Up/PageUp can increase scroll_offset without clamp, while render computes actual max scroll from current lines/viewport; when offset exceeds max by a large amount, Down/PageDown appear stuck until many decrements
	implication: root cause is unbounded scroll offset state; fix should clamp offset to computed max and reset to bottom on submit

- timestamp: 2026-03-30T12:22:10+05:30
	checked: agent-rs/src/tui.rs scroll-state patch
	found: draw path now records max_scroll_offset and clamps scroll_offset, Up/PageUp are capped to that max, submit_input resets offset to 0, and ClearHistory resets scroll state
	implication: no more oversized offset ranges; new prompts automatically return viewport to latest content

- timestamp: 2026-03-30T12:23:10+05:30
	checked: task fmt-test-after-real-patch (`cd agent-rs && cargo fmt && cargo test`)
	found: full suite passed with 30/30 tests, including new tui scroll regression tests
	implication: scroll fix compiles cleanly and is stable under automated verification

- timestamp: 2026-03-30T12:30:30+05:30
	checked: human verification checkpoint response
	found: user confirmed fixed in real usage (`confirmed fixed`)
	implication: fix is validated end-to-end and session can be archived


## Resolution
<!-- OVERWRITE as understanding evolves -->

root_cause: Prior three-layer chain is resolved; remaining issue was TUI navigation state where scroll_offset was not bounded to render-computed maximum and was not reset on submit, causing apparent down-scroll lockout for newer responses.
fix: Added max_scroll_offset tracking in draw, clamped scroll_offset to that bound, capped Up/PageUp increments to the current maximum, and reset scroll state on submit/clear so latest responses are reachable immediately.
verification: Automated verification passed via cargo fmt + cargo test (30/30, including new tui scroll tests), and user confirmed end-to-end fix in interactive TUI flow.
files_changed: [agent-rs/src/main.rs, agent-rs/src/tui.rs]
