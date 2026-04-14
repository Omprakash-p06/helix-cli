# Phase 22: Onboarding and Session Resilience - Research

**Researched:** 2026-04-13
**Domain:** First-run onboarding, crash-safe session persistence, explicit session lifecycle commands
**Confidence:** HIGH (implementation gaps and integration points), MEDIUM (best UX prompts wording)

## Summary

Phase 22 should convert the current "interactive startup + ephemeral TUI state" behavior into a durable session model with guided first-run onboarding, automatic crash-safe autosave, and explicit save/load/resume commands.

The codebase already has strong foundations:
- `start.py` provides model/interface/mode prompts and safe launch defaults.
- `agent-rs/src/input.rs` already persists readline command history to `~/.helix_history`.
- TUI command metadata already includes `/save` and `/load` in `agent-rs/src/tui/commands.rs`.

However, requirements are not fully met yet:
- `SETUP-02` requires guided first-run onboarding in CLI/TUI; current prompts run every launch and are not profile-backed.
- `SESSION-01` requires automatic crash-safe persistence and startup resume prompt; no durable chat session backend is wired.
- `SESSION-02` requires explicit `/save`, `/load`, `/resume`; command metadata exists, but runtime dispatch handles only `/clear` and `/mode`.

Primary recommendation:
1. Add a dedicated Rust session persistence module (`session.rs`) with atomic writes and schema versioning.
2. Wire autosave checkpoints into the existing TUI/LLM loop after each state-changing step.
3. Implement `/save`, `/load`, `/resume` command dispatch in `main.rs` and command palette action mapping in `tui/commands.rs`.
4. Add a first-run profile and onboarding flow (safe defaults + quick tour) that runs once, then allows explicit re-run.

## Standard Stack

### Core (Use)
| Library / Module | Purpose | Why |
|---|---|---|
| Rust `serde` + `serde_json` | Session serialization | Already used heavily across `agent-rs`; no new dependency |
| Rust `std::fs` + atomic temp rename pattern | Crash-safe persistence | Supports durable writes without partial file corruption |
| Existing `start.py` prompts | Onboarding entrypoint | Already collects model/interface/mode and can host first-run gate |
| Existing TUI event/action pipeline | Runtime integration | Clean place to trigger autosave and resume prompts |

### Optional (Only if needed)
| Library | Use case |
|---|---|
| `directories` or `platformdirs` for Rust | Standard cross-platform session storage location |
| `chrono` | Human-readable timestamps in session metadata |

## Architecture Patterns

### Pattern 1: Versioned Session Envelope

**What:** Persist all resumable runtime state in a versioned JSON envelope.

Recommended shape:
- `version`: schema version (start at 1)
- `created_at`, `updated_at`
- `model_name`, `exec_mode`, `ui_mode`
- `messages`: persisted chat messages (system/user/assistant/tool)
- `context_files`: optional context entries for TUI sidebar
- `token_snapshot`: tokens used / max tokens

**Why:** Enables forward-compatible evolution and deterministic load behavior.

### Pattern 2: Atomic Autosave Contract

**What:** Save on meaningful checkpoints, not only on clean exit.

Recommended checkpoints:
1. After user message is appended to `messages`.
2. After assistant response is finalized (`ResponseDone`).
3. Before graceful quit.

Write contract:
1. Serialize to `session.tmp`.
2. Flush/sync file.
3. Rename to target (`session.latest.json`).
4. Keep optional rolling backup (`session.prev.json`).

**Why:** Prevents losing an entire conversation on process crash or terminal kill.

### Pattern 3: Startup Resume Gate

**What:** At TUI boot, detect latest autosave and offer resume prompt.

Prompt behavior:
- If latest autosave exists and is valid: prompt `Resume previous session? [Y/n]`.
- If corrupted: show warning and offer fresh session.
- If none: start normally.

**Why:** Fulfills `SESSION-01` and keeps recovery low-friction.

### Pattern 4: Explicit Session Lifecycle Commands

**What:** Implement `/save`, `/load`, `/resume` as runtime commands, not just command metadata.

Required wiring:
- `tui/commands.rs`: map `save`, `load`, `resume` into `TuiAction::SystemCommand(...)`.
- `main.rs` TUI command handler: parse args and call persistence API.
- TUI feedback: emit `TuiEvent::SystemMessage` with success/failure details.

**Why:** `SESSION-02` requires verified command behavior. Current command catalog alone is insufficient.

### Pattern 5: One-Time Onboarding Profile

**What:** Separate first-run onboarding from every-run launch prompts.

Recommended behavior:
- First run: guided quick tour (mode/interface meaning, safe defaults, where data is stored).
- Persist onboarding completion and selected defaults to profile.
- Later runs: use profile defaults automatically, allow explicit override command/flag.

**Why:** Meets `SETUP-02` without adding repeated prompt friction.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Session persistence | ad hoc string dumps without schema | typed session structs + serde JSON | Safer loads, easier migrations |
| Crash handling | write directly to final file | temp file + rename atomic write | Prevents partial/corrupt session files |
| Command support | only command palette metadata | metadata + runtime dispatcher implementation | Commands must be executable |
| Onboarding state | implicit assumptions in code | explicit onboarding/profile file | Deterministic first-run behavior |
| Resume logic | silent auto-restore with no user consent | startup prompt + clear fallback path | Avoids surprising session restores |

## Common Pitfalls

### 1) Treating command history as session persistence
`input.rs` persists CLI history (`~/.helix_history`), but that is not chat state recovery.

### 2) Advertising commands without handlers
`/save` and `/load` appear in command definitions but are not dispatched in current `execute_command`/`main.rs` handling paths.

### 3) Saving only on clean exit
A crash-safe requirement cannot rely on graceful termination hooks alone.

### 4) Overwriting session file in-place
In-place write can leave truncated JSON on crash; always use temp + atomic rename.

### 5) Coupling onboarding to every launch
Current startup prompts are useful but do not represent a first-run completion state.

## Source Evidence

- `start.py` prompts for model/interface/mode every launch and does not persist onboarding completion/profile.
- `agent-rs/src/input.rs` persists readline history to `~/.helix_history` via `save_history`, but no session snapshot backend exists there.
- `agent-rs/src/tui/commands.rs` defines `/save` and `/load` commands in `default_commands()`.
- `agent-rs/src/tui/commands.rs` `execute_command()` maps only `help`, `clear`, `agent`, `chat`, and `quit`.
- `agent-rs/src/main.rs` TUI `SystemCommand` handler currently implements `/clear` and `/mode...` branches, with no `/save` `/load` `/resume` branch.

## Implementation Readiness

Ready for `/gsd-plan-phase 22`.

Recommended planning order:
1. Define session persistence contracts (schema, storage path, atomic IO helpers, migration/version strategy).
2. Implement autosave/resume flow in Rust TUI runtime and startup prompt handling.
3. Implement `/save`, `/load`, `/resume` command parsing + command palette mappings + user feedback events.
4. Add onboarding profile + first-run quick tour (CLI/TUI safe defaults) and tests for first-run vs returning-user paths.

## Suggested Verification Targets (for planning)

- Automated:
  - Rust unit tests for session serialize/deserialize, corrupted file recovery, atomic write behavior.
  - Rust integration test for command dispatcher (`/save`, `/load`, `/resume`) success/failure paths.
  - Python tests for first-run onboarding profile creation and subsequent default loading.
- Manual:
  - Start a chat, kill process mid-session, relaunch, verify resume prompt and restored transcript.
  - Run first launch with no profile, confirm guided onboarding and safe defaults.
