# Phase 27: TUI Animations & Settings Panel - Research

**Researched:** 2025-04-23
**Domain:** Terminal User Interface (Rust/Ratatui), Animations, State Management
**Confidence:** HIGH

## Summary

This research investigates the optimal implementation path for Phase 27 of the Helix Agent, focusing on high-quality terminal animations and a robust settings modal. The project currently uses `ratatui` (v0.26) with an event-driven loop that redraws only on input or data events. 

To support smooth animations (fades, pulses, and spinners), the architecture must transition to a hybrid tick/event loop (approx. 30 FPS). The state-of-the-art approach for Ratatui animations is to use the `tachyonfx` library for buffer-level effects like fades and hue shifts, and `throbber-widgets-tui` for indeterminate loading indicators. The settings modal will follow the standard "List + Detail" pattern, centered using existing layout utilities.

**Primary recommendation:** Use `tachyonfx` (v0.11) for fades and pulsing effects, and `throbber-widgets-tui` (v0.11) for the thinking spinner. Upgrade the `run_tui` loop to include a `tokio::time::interval` tick at ~33ms.

## User Constraints (from CONTEXT.md)

*No CONTEXT.md found for this phase. Research is based on REQUIREMENTS.md and existing codebase.*

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TUI-06 | Throbber/spinner animation for "thinking" | `throbber-widgets-tui` provides a mature, stateful spinner widget. |
| TUI-07 | Fade-in effect for new messages (150ms) | `tachyonfx::fx::fade_from` can interpolate colors on the buffer. |
| TUI-08 | Pulsing status for tool execution | `tachyonfx::fx::ping_pong` + `fx::fade_to_fg` creates smooth pulsing. |
| TUI-09 | Animated token usage gauge | Manual interpolation of values in the tick loop (tweening). |
| TUI-10 | Settings modal | Layout-based centering with `Clear` and `List` widgets. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ratatui` | 0.26.0 | TUI Framework | Existing project standard. |
| `tachyonfx` | 0.11.1 | Shader-like effects | SOTA for fades, pulses, and color-space animations in Ratatui. [VERIFIED: GitHub/Docs] |
| `throbber-widgets-tui` | 0.11.0 | Spinner widgets | De-facto standard for indeterminate progress in Ratatui. [VERIFIED: Docs.rs] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| `tokio` | 1.43 | Async Runtime | Existing project standard for the event loop. |

**Installation:**
```bash
# Note: Ensure versions match Ratatui 0.26 compatibility
cargo add tachyonfx@0.11.1
cargo add throbber-widgets-tui@0.11.0
```

## Architecture Patterns

### Hybrid Tick/Event Loop
The current loop in `agent-rs/src/tui.rs` waits on `event::poll` or `event_rx.recv()`. For animations, it must trigger a redraw periodically even when no external events occur.

```rust
// Pattern: Animation-aware event loop
let mut interval = tokio::time::interval(Duration::from_millis(33)); // ~30 FPS
let mut last_tick = Instant::now();
let mut effect_manager = tachyonfx::EffectManager::default();

loop {
    let dt = last_tick.elapsed();
    last_tick = Instant::now();
    
    terminal.draw(|f| {
        draw(f, &mut app, &mut effect_manager, dt);
    })?;

    tokio::select! {
        _ = interval.tick() => {
            app.on_tick(); // Advance spinners and gauges
        }
        // ... handle other events ...
    }
}
```

### Pattern 1: Buffer-Level Post-Processing (Animations)
**What:** Render widgets normally, then apply visual effects to the buffer before it is flushed to the terminal.
**When to use:** Fades (`TUI-07`), Pulsing (`TUI-08`).
**Example:**
```rust
// Source: https://github.com/ratatui/tachyonfx
let fade = fx::fade_from(Color::Black, Color::White, (150, Interpolation::QuadOut));
effect_manager.add_effect(fade, message_area);
```

### Pattern 2: Stateful Widget Ticking (Spinners)
**What:** Store a `ThrobberState` in the app and call `calc_next()` on every tick.
**When to use:** Thinking state (`TUI-06`).

### Anti-Patterns to Avoid
- **Blocking `event::read`:** Never use blocking reads in an animation loop; it will freeze the UI until input is received.
- **Hand-rolling Fades:** Interpolating RGB values manually for every cell is error-prone and slow; use `tachyonfx`.
- **Global Redraw on Every Byte:** Ensure the tick rate is capped (e.g., 30 FPS) even if tokens arrive faster, to prevent high CPU usage.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Color Interpolation | Manual RGB math | `tachyonfx` | Handles HSL shifts and true-color fallbacks efficiently. |
| Spinner Sequences | `vec!["-", "\\", "|", "/"]` | `throbber-widgets-tui` | Provides multiple sets (dots, clocks, braille) and handles state correctly. |
| Modal Centering | Manual Rect math | `centered_rect` (exist) | Centering on terminal resize is tricky with manual math. |

## Common Pitfalls

### Pitfall 1: Frame Delta Accumulation
**What goes wrong:** Animations run at different speeds on different hardware or during lag.
**Why it happens:** Using a fixed increment per frame instead of using the time delta (`dt`) since the last frame.
**How to avoid:** Always pass `dt` to animation managers and interpolation functions.

### Pitfall 2: Ghosting on Scroll
**What goes wrong:** A fade-in effect is applied to a static area, but the message scrolls away while fading.
**Why it happens:** Effect areas are not updated when the underlying content moves.
**How to avoid:** In Phase 27, apply fade-ins to the *newly rendered* message at its current position, or use short durations (150ms) to minimize visible ghosting during active scrolling.

## Code Examples

### Animated Token Gauge (Tweening)
```rust
// In TuiApp::on_tick
fn on_tick(&mut self) {
    let target = self.current_tokens as f32;
    let delta = (target - self.animated_tokens) * 0.2; // Smooth damping
    self.animated_tokens += delta;
    
    if (self.animated_tokens - target).abs() < 0.1 {
        self.animated_tokens = target;
    }
}
```

### Pulsing Status (tachyonfx)
```rust
// In TuiApp
let pulsing_tool = fx::repeating(
    fx::ping_pong(
        fx::fade_to_fg(Color::Yellow, (800, Interpolation::SineInOut))
    )
);
```

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| True Color | `tachyonfx` | ✓ | — | 256-color / 16-color interpolation |
| Rust Toolchain| Compilation | ✓ | 1.95.0 | — |
| Ratatui | Framework | ✓ | 0.26 | — |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | native rust (cargo test) |
| Config file | none |
| Quick run command | `cargo test -p agent-rs --lib tui` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| TUI-09 | Gauge value interpolates | unit | `cargo test test_gauge_interpolation` |
| TUI-10 | Settings state updates | unit | `cargo test test_settings_toggle` |

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Sanitize theme paths and model names in settings. |
| V4 Access Control | yes | Settings modal "Permissions" toggle must sync with orchestrator policy tiers. |

### Known Threat Patterns for Ratatui

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path Traversal | Information Disclosure | Hardcode config directory (`~/.config/helix-agent/`) for theme loading. |
| Privilege Escalation | Elevation of Privilege | Ensure TUI "Permissions" setting only *requests* tiers; orchestrator *enforces* them. |

## Sources

### Primary (HIGH confidence)
- [Official TachyonFX Docs](https://github.com/ratatui/tachyonfx) - Effect patterns and Ratatui 0.26 compatibility.
- [Throbber-widgets-tui Docs](https://docs.rs/throbber-widgets-tui) - State management for spinners.
- [Ratatui official examples](https://ratatui.rs/examples/) - Modal and layout patterns.

### Secondary (MEDIUM confidence)
- Web search for "ratatui animation patterns" - hybrid loop consensus.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Libraries are mature and verified compatible with 0.26.
- Architecture: HIGH - Standard "tick" pattern for Ratatui animations.
- Pitfalls: MEDIUM - Scroll/animation interaction is a known challenge in TUIs.

**Research date:** 2025-04-23
**Valid until:** 2025-05-23
