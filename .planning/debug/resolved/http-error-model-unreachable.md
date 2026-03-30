# Debug Session: http-error-model-unreachable

status: resolved
started_at: 2026-03-30T00:36:00+05:30

## Symptoms (prefilled)
- expected: model should return a response in TUI
- actual: HTTP error shown in TUI instead of model response
- errors: `error sending request for url (http://127.0.0.1:8080/v1/chat/completions)`
- timeline: began around ~7pm
- reproduction: not deterministic

## Investigation
- Verified server/runtime state and log evidence.
- Found historical CUDA OOM crashes in `logs/start_server.stderr.log`.
- Confirmed endpoint can recover and serve when server is online.
- Identified fragile behavior: a single transient connect/timeout error immediately failed the user prompt.
- Identified weak diagnostics: GPU layer hint reported `unset` when env vars were absent, even though `scripts/config.py` had configured values.

## Root Cause
1. Intermittent backend availability (including OOM-induced restarts/crashes) created transient HTTP connect/timeout failures.
2. Chat requests were still vulnerable to startup race and fallback timing, so the first prompt could fail before the model server settled.
3. In chat mode, the model emitted long `reasoning_content` streams by default, which caused poor UX and apparent hangs for short prompts.

## Fix Implemented
- Added robust recovery helpers in Rust (`send_with_retry`, `send_with_recovery`, `send_with_forced_retry`) and wired both terminal/TUI request paths.
- Added model-server auto-boot path from Rust with readiness probing and forced safe launch env (`HELIX_BACKEND_HINT=cpu`, `HELIX_GPU_LAYERS=0`) during recovery.
- Added runtime env overrides in `scripts/start_server.py` so Rust can force backend/layer behavior for recovery startup.
- Added explicit fallback defaults in `scripts/config.py` (`FALLBACK_GPU_LAYERS=0`, `FALLBACK_BACKEND_HINT="cpu"`).
- Forced chat-mode request payload to disable thinking via `chat_template_kwargs: {"enable_thinking": false}`.
- Added non-empty fallback user message when no visible content is produced.
- Added chat token budget helper (`chat_max_tokens`) to keep chat responses bounded.

## Validation
- `cargo fmt && cargo test` passed (17 tests).
- Manual verification from user terminal:
	- `cargo run --quiet -- --prompt "hey"` returns visible response (`Hey! How can I help you today?`).
	- `cargo run --quiet -- --prompt "hello, how are you?"` returns visible response.
- Cold-start verification (server stopped first) passed for both:
	- `--prompt hey`
	- `--prompt "Reply with exactly OK"`

## Follow-up
- If errors persist with OOM evidence, reduce `GPU_LAYERS` in `scripts/config.py` and/or set `FALLBACK_GPU_LAYERS` explicitly.
