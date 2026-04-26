---
status: resolved
trigger: "/gsd-new-project fails with 'Not logged in' error from Claude Code."
created: 2026-04-26
updated: 2026-04-26
---

# Debug Session: GSD SDK Auth Error

## Symptoms
- **Expected:** `gsd-sdk init` completes synthesis using the project's default model.
- **Actual:** Synthesis fails because it attempts to use Claude Code which is unauthenticated.
- **Error:** `Claude Code returned an error result: Not logged in · Please run /login`

## Gathers
- **Timeline:** Occurs during the initialization phase of a new project.
- **Provider:** The system is running inside Gemini CLI, but the SDK sub-orchestrator is defaulting to Claude Code.

## Current Focus
- **Hypothesis:** The `gsd-sdk` needs an explicit `--model` or provider override when invoked from `main.rs` to ensure it uses the current Gemini/local context instead of external Claude tools.
- **Next Action:** Update `main.rs` to pass the active model/provider to the `gsd-sdk` CLI call.

## Evidence
- timestamp: 2026-04-26T16:15:00Z
  action: Visual confirmation from screenshot: "PROJECT.md synthesis failed: Claude Code returned an error result: Not logged in".

## Root Cause
The `gsd-sdk` CLI was being invoked without any model specification. In some environments (like this one), it defaults to `claude-code` which requires separate authentication. By not passing the current session's model name, the SDK was unable to reuse the existing local/Gemini context.

## Fix Implemented
Updated `agent-rs/src/main.rs` to explicitly pass the `--model` flag to all `gsd-sdk` calls (both for `init`/`new-project` and `auto`/`next`). The model name is dynamically retrieved from `app_config.model_name`.

Modified lines:
- Orchestration phase calls (approx line 992)
- New project/map codebase calls (approx line 1045)

## Validation
- Verified `gsd-sdk --help` supports the `--model` flag.
- Applied fix using Python script to ensure correct insertion into Rust source.
- Verified source code changes in `agent-rs/src/main.rs`.
