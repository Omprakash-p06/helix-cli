---
phase: 19-implement-terminal-ui-features
plan: 01
subsystem: tui
tags: [tui, layout, contract, sidebar, status-bar]
requires: []
provides: [TuiLayoutMode, ContextSnapshot, sidebar-toggle, status-hud]
affects: [agent-rs/src/tui.rs, agent-rs/src/tui/api.rs, agent-rs/src/tui/state.rs, agent-rs/src/tui/themes.rs, agent-rs/src/tui/commands.rs, agent-rs/src/tui/events.rs, agent-rs/src/main.rs]
tech-stack:
  added: []
  patterns: [three-region-layout, layout-mode-dispatch, sidebar-toggle]
key-files:
  created: []
  modified:
    - agent-rs/src/tui.rs
    - agent-rs/src/tui/api.rs
    - agent-rs/src/tui/state.rs
    - agent-rs/src/tui/themes.rs
    - agent-rs/src/tui/commands.rs
    - agent-rs/src/tui/events.rs
    - agent-rs/src/main.rs
key-decisions:
  - "Moved ThemeName to state.rs and theme presets/rendering to themes.rs for single responsibility"
  - "Status bar uses exact format: model: {name} | {mode} mode | {status} ... [{current}/{max} tok] | {connection}"
  - "Sidebar hidden when terminal width < 60 chars to avoid cramped layouts"
requirements-completed: [TUI-01]
duration: "12 min"
completed: "2026-04-08"
---

# Phase 19 Plan 01: TUI Foundation Summary

Replaced the legacy single-pane TUI with a three-region layout-aware shell plus shared telemetry contract types.

## Duration & Scope
- **Duration:** ~12 min
- **Tasks:** 2 completed
- **Files:** 7 modified

## Task 1: Normalize the shared TUI launch and telemetry contract
- Added `TuiLayoutMode` (Wide/Compact), `ConnectionState`, `ContextFileEntry`, `SessionInfo`, `StatusBanner`, `StatusLevel` to `tui/api.rs`
- Extended `TuiAction` with 14 new variants (sidebar toggle, command palette, theme, scroll, layout)
- Extended `TuiEvent` with `ThemeChanged` and `ContextSnapshot` structured variants
- Updated `tui/state.rs` with layout mode, sidebar tab, tool timeline, collapsed tools, status banner
- Updated `tui/commands.rs` with Phase 19 spec slash commands and fuzzy filtering by example text
- Updated `tui/themes.rs` with NO_COLOR monochrome, custom config file parsing, hex color parser
- Wired submodules (api, commands, events, state, themes) into tui.rs module tree
- Parsed `--layout wide|compact` from CLI args in main.rs
- **Commit:** `4c9466f`

## Task 2: Replace the root frame with the three-region shell and live HUD
- Refactored `draw()` to split top row into conversation + optional sidebar
- Added `draw_sidebar()` rendering Session Info, Token Usage bar, and keyboard shortcuts
- Updated status bar to show `model:`, `mode`, `[current/max tok]`, connection state
- Wired `Ctrl+B` to toggle sidebar visibility without touching backend state
- Passed `layout_mode` through `run_tui()` to `TuiApp`
- Updated `ContextSnapshot` handler to populate model/mode/connection labels
- **Commit:** `938ba92`

## Verification
- `cargo check` passes with 0 warnings
- `cargo test -q` passes — all 68 tests green

## Deviations from Plan

**[Rule 3 - Blocking] Fix KeyModifiers::Default in events.rs tests**
- Found during: Task 2 verification
- Issue: `Default::default()` not implemented for `KeyModifiers` in current crossterm version
- Fix: Replaced with `KeyModifiers::NONE`
- Files: `agent-rs/src/tui/events.rs`

**Total deviations:** 1 auto-fixed. **Impact:** None — test-only fix.

## Next Phase Readiness
Ready for Plan 19-02 (slash command palette, ghost autocomplete, token HUD, themes).
