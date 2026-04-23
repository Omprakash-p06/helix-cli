# Phase 27: TUI Animations & Settings Panel - Validation

This document defines the validation criteria for Phase 27, ensuring that animations are smooth and the settings panel is functional and persistent.

## Must-Have Truths (Observable Behaviors)

### TUI-06: Thinking Spinner
- [ ] When the model is in a "thinking" state (generating but no tokens yet), a spinner is visible.
- [ ] The spinner animates smoothly at ~30 FPS.
- [ ] The spinner disappears once content streaming begins.

### TUI-07: Message Fade-in
- [ ] New chat messages appear with a smooth fade-in effect.
- [ ] The fade duration is approximately 150ms.
- [ ] Fade effects do not leave "ghost" artifacts when scrolling.

### TUI-08: Pulsing Status
- [ ] Tool execution entries in the sidebar pulse between two colors (e.g., standard foreground and yellow).
- [ ] Pulsing is continuous while the tool is `Running`.

### TUI-09: Animated Token Gauge
- [ ] The token usage gauge bar slides smoothly to new values rather than jumping.
- [ ] Large jumps in token usage (e.g., /clear) result in a visible "draining" animation.

### TUI-10: Settings Modal
- [ ] Pressing `Ctrl+S` opens the settings modal.
- [ ] The modal is centered and opaque (using `Clear`).
- [ ] Themes can be changed and applied instantly.
- [ ] Settings (Theme, Mode, Model) are persisted to `settings.json` and survive a restart.

## Automated Verification

| Req ID | Test Name | Command |
|--------|-----------|---------|
| TUI-09 | test_gauge_interpolation | `cargo test -p agent-rs tui::state::tests::test_gauge_interpolation` |
| TUI-10 | test_settings_persistence | `cargo test -p agent-rs tui::state::tests::test_settings_persistence` |
| TUI-10 | test_settings_toggle | `cargo test -p agent-rs tui::state::tests::test_settings_toggle` |

## Manual Verification (UAT)

1. **Animation Smoothness**:
   - Run the agent with `HELIX_UI_MODE=tui`.
   - Verify that the UI remains responsive and animations are fluid (no stuttering).
   - Check CPU usage to ensure it doesn't spike excessively due to the 30 FPS tick loop.

2. **Settings Persistence**:
   - Open settings (`Ctrl+S`).
   - Change the theme to a high-contrast theme.
   - Close the agent.
   - Restart the agent and verify the high-contrast theme is still active.

3. **Tool Pulsing**:
   - Run a tool-intensive prompt (e.g., "Analyze the files in the current directory").
   - Watch the sidebar and verify tool entries pulse while active.

## Performance Targets
- Frame rate: 30 FPS (+/- 2 FPS).
- CPU overhead: < 5% additional CPU usage compared to non-animated TUI.
- Fade-in latency: 150ms fixed duration.
