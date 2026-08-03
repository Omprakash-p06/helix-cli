# Codebase Concerns

**Analysis Date:** 2026-08-03

## Tech Debt

**Two divergent `scripts/config.py` generators (setup-time regression):**
- Issue: `setup.py` (`generate_config()`, `setup.py:1024-1074`) overwrites `scripts/config.py` with a minimal file that defines only constants (`MODEL_NAME`, `MODEL_PATH`, `BASE_URL`, `GPU_LAYERS`, `CONTEXT_SIZE`, `DANGEROUS_COMMANDS`, ...). The checked-in `scripts/config.py` (439 lines) is a much richer module exposing `scan_models_directory()` (used by `start.py:25`), `build_model_entry()` / `DETECTED_VRAM_GB` (used by `start.py` and the Rust config bridge `agent-rs/src/config.rs:65`), `MODEL_CATALOG`, `AUDIT_ENABLED`, `TOOL_PERMISSION_TIER`, `CHAT_SYSTEM_PROMPT`.
- Why: Two separate code paths grew independently — one generator in `setup.py` and a hand-maintained module.
- Impact: Every fresh `python setup.py` run replaces the rich module with the minimal one; then `start.py` crashes with `AttributeError: module 'config' has no attribute 'scan_models_directory'` and the Rust agent's `AppConfig::load_from_python()` (`agent-rs/src/config.rs:43-103`) fails at `config.build_model_entry(...)`, breaking the whole stack after setup.
- Fix approach: Delete `generate_config()` from `setup.py` and have setup only mutate constants in the canonical `scripts/config.py` (or have `generate_config` emit the full schema).

**`run_terminal_command` has two divergent implementations, one of which is dead code:**
- Issue: `agent-rs/src/agent_core/tool_runtime.rs:239-240` intercepts `run_terminal_command` and always routes to `execute_sandboxed_command` (Docker). The registry implementation `RunTerminalCommandTool` → `execute_run_terminal_command()` in `agent-rs/src/tools.rs:491-569`, which executes directly on the host via `sh -c` / `cmd /C`, is unreachable.
- Why: Sandboxing was bolted on without removing the old path.
- Impact: The host-execute path is a latent bypass — any future code path that calls the registry directly (or changes the intercept condition) silently switches to un-sandboxed host execution with weaker checks (no allowlist, no path canonicalization).
- Fix approach: Delete `execute_run_terminal_command` from `tools.rs` (or make the registry entry a thin wrapper over the sandbox path) so there is exactly one execution path.

**Inert `sandbox_interpreters` config flag:**
- Issue: `AppConfig.sandbox_interpreters` is defined with default `true` (`agent-rs/src/config.rs:21-29`) but never read anywhere in `agent-rs/src`.
- Why: Designed for a "sandbox interpreters but not plain commands" mode that was never implemented.
- Impact: Users cannot opt into/out of sandboxed terminal execution; the flag silently does nothing while implying configurability.
- Fix approach: Wire it into `tool_runtime.rs` (when false, allow host execution with policy checks) or remove the field.

**Duplicated security policy logic across files:**
- Issue: `blocked_command_reason` / `command_matches_block_pattern` are implemented in `agent-rs/src/security/policy.rs:176-233` AND again as `is_blocked_command` in `scripts/config.py:165-188` (plus a third copy in `scripts/config.py` `BLOCKLIST`). The dangerous-command lists also diverge: `policy.rs:141-143` = `["rm","dd","mkfs","fdisk","shutdown","reboot","sudo"]` vs `scripts/config.py:439` = `["rm","mv","chmod","dd","mkfs","fdisk","systemctl","reboot","shutdown"]`.
- Why: Each layer (Python config, Rust policy, TUI approval) re-implemented the blocklist independently.
- Impact: `mv`/`chmod`/`systemctl` are flagged dangerous in config but not in the Rust policy that actually gates execution; blocklist additions must be made in multiple places and will drift.
- Fix approach: Single source of truth (Rust `policy.rs`) exported to Python, or vice versa; have the agent read `dangerous_commands` from config into the policy engine instead of hardcoding.

**Monolithic `main.rs` and `tui.rs`:**
- Issue: `agent-rs/src/main.rs` is 2208 lines and `agent-rs/src/tui.rs` is 2077 lines; `main.rs` embeds the full agent loop, retry/OOM heuristics, `/gsd` slash-command dispatch, and gsd-sdk spawning inline.
- Why: Iterative feature addition without decomposition.
- Impact: High change-collision risk; the four `gsd-sdk` spawn sites (`main.rs:982, 1039, 1094, 1117`) are near-duplicates that `apply_fix.py` already had to patch textually.
- Fix approach: Extract gsd-sdk invocation into one helper in `agent_core`; split the agent loop into `agent_core/loop.rs`.

**One-off `apply_fix.py` committed at repo root:**
- Issue: `apply_fix.py` is a throwaway script that edits `agent-rs/src/main.rs` by string-matching 12-line code patterns.
- Why: Applied a patch (adding `--model` args to gsd-sdk calls) without a proper PR.
- Impact: If re-run, it will corrupt `main.rs` (patterns may no longer match or may match unexpected locations); it is not idempotent and has no tests.
- Fix approach: Delete the file; the fix is already in `main.rs` (the `--model` args are present).

**Rust agent depends on Python at runtime:**
- Issue: `AppConfig::load_from_python()` (`agent-rs/src/config.rs:43-137`) shells out to `python -c` to import `scripts/config.py` and print JSON. The Rust binary is not self-contained.
- Why: Config lives in Python for the launcher/server stack.
- Impact: If `python` is not on PATH (or config.py imports fail), the agent cannot start; config schema changes in Python silently break the Rust agent with an opaque "Failed to parse JSON config" error.
- Fix approach: Generate a JSON config file once at setup time (`scripts/config.json`) and read it directly from Rust.

**Empty `llama.cpp/` directory tracked in git:**
- Issue: `llama.cpp/` is an empty directory in the repo; `setup.py:498-503` deletes and re-clones `https://github.com/ggerganov/llama.cpp.git` at HEAD when `CMakeLists.txt` is absent. `.gitignore` only ignores `llama.cpp/build/`, not the source.
- Why: Initial attempt to vendor the dependency was abandoned.
- Impact: First-run setup downloads a moving target; builds are non-reproducible and can break when upstream llama.cpp changes flags/APIs; the empty tracked dir is confusing.
- Fix approach: Pin a commit (clone `--branch` / `--depth` at a known SHA) or make it a proper submodule; record the pin in `scripts/config.py`.

**Static model catalog references unverifiable/fictional model IDs:**
- Issue: `scripts/config.py:191-335` (`MODEL_CATALOG`) and `scripts/download_model.py:36-55` reference models like `Gemma-4-E4B`, `Qwen-3.6-27B-MoE`, `Qwen-3.6-35B-MoE` (with `jinja`/`mmproj` handling), while `scripts/model_install.py:11-46` (`TRUSTED_MODELS`) marks the Qwen entries `verification_status: "blocked-until-pinned"` with `sha256: None`.
- Why: Catalog was written ahead of verification.
- Impact: `setup.py` option 1/2 models are pinned+verified, but any default-model path that resolves to an unpinned catalog entry fails `validate_trusted_model_spec()` (`model_install.py:77-87`) — see Known Bugs.
- Fix approach: Reconcile the two model registries; pin revisions + SHA-256 for every installable entry or remove unpinned entries from the catalog.

## Known Bugs

**setup.py overwrites scripts/config.py with a broken minimal config (see Tech Debt #1):**
- Symptoms: After `python setup.py`, `start.py` fails immediately: `AttributeError: module 'config' has no attribute 'scan_models_directory'`; the Rust agent then fails config load (`config.rs` bridge calls `config.build_model_entry` which does not exist).
- Trigger: Run `python setup.py` on a fresh clone (or any machine where `scripts/config.py` is generated).
- Workaround: Restore the rich `scripts/config.py` from git after setup.
- Root cause: Two divergent config generators.
- Blocked by: None — fix is straightforward.

**`run_terminal_command` is effectively non-functional:**
- Symptoms: Tool always returns failure. Either "Sandbox initialization failed" (Docker daemon/CLI absent) or, when Docker is present, `git: not found` / `python: not found` style errors inside the container.
- Trigger: Any terminal-command tool call in agentic mode.
- Root cause 1: `tool_runtime.rs:297` requires `DockerSandbox::new()` → `Docker::connect_with_local_defaults()`; no host fallback (the working host path in `tools.rs:491` is dead code).
- Root cause 2: The sandbox image is hardcoded to `alpine:3.20` (`tool_runtime.rs:313`), which does not contain most allowlisted binaries (`cargo`, `npm`, `node`, `python`, `pytest`, `rg`, `sed`, `awk` — even `git` is not installed by default in the alpine image).
- Workaround: None; agents cannot run build/test/dev commands.

**`max_tokens: 8192` hardcoded breaks small-context setups:**
- Symptoms: On tier-1/2 hardware (context 2048/4096 per `scripts/system_check.py` TIER_CONFIGS), agentic-mode requests are rejected by llama-server ("requested tokens exceed context") in both terminal (`main.rs:1280, 1757`) and web (`server.rs:229`) modes.
- Trigger: Run agentic mode with `CONTEXT_SIZE <= 8192` and ask the model to produce tool calls; the server rejects `max_tokens=8192`.
- Workaround: Raise context size, or use chat mode (chat clamps to `HELIX_CHAT_MAX_TOKENS`, default 1024, `main.rs:331-336`).
- Root cause: Hardcoded value instead of `min(app_config.context_size, 8192)`.

**Universal HuggingFace model install in setup.py always fails:**
- Symptoms: Choosing option 3 ("Any HuggingFace GGUF") in `setup.py:484-486` → "Failed to install model" → `sys.exit(1)`.
- Trigger: `python setup.py`, pick 3, pick any repo/file.
- Root cause: `install_model_spec()` (`model_install.py:168-177`) calls `validate_trusted_model_spec()` which raises for any spec without a pinned `revision` + 64-char `sha256`; universal downloads are unpinned by design.
- Workaround: Use `scripts/download_model.py` (which verifies against HF LFS metadata) instead.

**Port 3000 hardcoded with no conflict handling:**
- Symptoms: Web mode fails to start if port 3000 is occupied; the process panics (`server.rs:107-108` `.unwrap()` on bind).
- Trigger: Any other local service on 3000, then `HELIX_UI_MODE=web` via `start.py`.
- Root cause: `server.rs:99` hardcodes `let port = 3000;` and `web-ui/src/App.tsx:38` hardcodes `http://127.0.0.1:3000/chat`; neither reads `HELIX_SERVER_PORT`.
- Workaround: Kill the occupying process.

**setup.py token-speed benchmark can hang forever:**
- Symptoms: Setup hangs indefinitely at "Sending benchmark completion request (no timeout)...".
- Trigger: llama-server loads the model but never responds to `/v1/completions` (e.g., OOM thrash); `setup.py:749-752` issues `requests.post(...)` with no `timeout`.
- Root cause: The request deliberately has no timeout (comment at line 747) and `wait_for_server_with_process` is only used for readiness, not the completion call.
- Workaround: Kill setup; re-run.
- Fix: Add `timeout=` to the completion request and a total benchmark deadline.

**start.py kills unrelated processes:**
- Symptoms: On startup, `start.py:236-246` runs `taskkill /F /IM llama-server.exe` / `pkill -f "llama-server"` and the koboldcpp equivalents unconditionally.
- Trigger: Run `start.py` while any other project's llama-server/koboldcpp process is running (or a user process whose name matches).
- Root cause: Image-name/pattern kill without PID-file ownership checks (the `watchdog.rs` ownership machinery exists but is not used here).
- Workaround: Manually restart the killed processes.

## Security Considerations

**Unauthenticated, permissive-CORS agent API on localhost:**
- Risk: `agent-rs/src/server.rs:90-97` exposes `/chat` (SSE agent loop with tool execution), `/v1/tools`, `/v1/context`, `/v1/status` with `.layer(CorsLayer::permissive())` and no auth. Any webpage the user visits can `fetch("http://127.0.0.1:3000/chat", ...)` and drive the agent to call tools (write files, run sandboxed terminal commands, read workspace files). Classic local-service CSRF / DNS-rebinding vector; permissive CORS removes the browser same-origin barrier.
- Current mitigation: Binds to 127.0.0.1; policy engine gates tools (`tier_allows_tool`, `RequireApproval`); in web mode the permission requester is still wired (`main.rs:622-623`, since `exec_mode` is `agentic`/`chat`, not `server`) so `inquire` prompts appear in the terminal.
- Recommendations: Require a per-session bearer token (generated by `start.py`, passed to the web UI); restrict CORS to the Vite origin; validate `Origin`/`Host` headers (DNS-rebinding protection); add a rate limit on `/chat` (governor is already a dependency).

**Approval prompts surface in the terminal, not the browser:**
- Risk: In web mode, `InquirePermissionRequester` (`agent-rs/src/tui/approval.rs:26-31`) blocks on stdin of the terminal running the server. A browser-driven attacker can queue requests whose prompts the user may dismiss with Enter (approve) without reading.
- Current mitigation: `Confirm::new(...).with_default(false)` defaults to deny.
- Recommendations: In web mode, route approval through an SSE/browser channel or auto-deny with an explicit "denied — approve from terminal" message.

**Raw HTML rendering of LLM output (XSS):**
- Risk: `web-ui/src/App.tsx:163` renders assistant messages with `<ReactMarkdown rehypePlugins={[rehypeRaw]}>`. The agent's output — which includes content read from files and scraped web pages (`agent-rs/src/agent_core/web_research/...`) — is rendered as raw HTML. A malicious `<img onerror=...>` or `<script>` in a file/webpage the agent quotes executes in the browser context (which can reach the agent API and any cookies).
- Current mitigation: Web content is sanitized to markdown client-side (`sanitize.rs` strips script/style/iframe/form) before being fed to the LLM, but the LLM can echo/regurgitate HTML, and `rehypeRaw` re-enables it.
- Recommendations: Drop `rehype-raw` (or sanitize rendered HTML with DOMPurify); add `Content-Security-Policy` headers in the Vite dev server config.

**SSRF checks are bypassable and redirects are followed:**
- Risk: `is_ssrf_safe()` (`agent-rs/src/agent_core/web_research/sanitize.rs:21-32`) is a prefix regex on the lowercase URL; it misses alternate encodings (`http://2130706433/`, `http://0x7f000001/`, `http://0177.0.0.1/`, `http://[::ffff:7f00:1]/`). `reqwest_fetch` (`worker.rs:131-152`) builds a client with default redirect following (up to 10 redirects) — a public URL can redirect to `127.0.0.1`/cloud metadata.
- Current mitigation: Blocks the common literal forms; worker pool rate-limits to 2 req/s (`worker.rs:42`).
- Recommendations: Resolve DNS and validate all A records before fetching; disable redirects or re-validate each hop; block the full private range (including IPv6, `0.0.0.0`, and non-canonical encodings).

**Weak prompt-injection detection:**
- Risk: `is_prompt_injection_pattern` (`agent-rs/src/security/policy.rs:540-554`) matches 4 literal regexes against tool arguments only. File contents and tool outputs are injected into context largely unfiltered (only `</untrusted_web_content>` breakout tags are escaped, `sanitize.rs:69-71`).
- Current mitigation: The `untrusted_web_content` wrapper + breakout escaping for web content; `policy.rs` injection regex on tool args.
- Recommendations: Treat file reads as untrusted for directive content; consider instruction-hierarchy prompting and stronger injection heuristics before tool execution.

**Uncensored/"abliterated" default models:**
- Risk: `setup.py:28-39` ships `GPT-OSS-20B-...-abliterated-uncensored-...` and `Qwen3.5-9B-Uncensored-...-Aggressive` as the two default install options; an abliterated model paired with host-tooling raises the risk profile of autonomous tool-calling.
- Current mitigation: Policy engine + Docker sandbox + HITL approval.
- Recommendations: Offer a censored default first; keep uncensored options behind an explicit warning prompt.

## Performance Bottlenecks

**Context indexing at every agent start:**
- Problem: `main.rs:636-646` initializes the tree-sitter symbol index (`agent-rs/src/context/indexer.rs`, 537 lines) over the whole workspace on each startup, then builds a repo skeleton budgeted at 5k tokens (`main.rs:649`).
- Measurement: Not instrumented; startup cost scales with repo size (full-tree parse per file via tree-sitter, `indexer.rs:351-361`).
- Cause: `ContextEngine::initialize()` walks and parses all source files even if unchanged (incremental cache exists via `is_cached`, but the walk is unconditional).
- Improvement path: Skip the walk when `helix_context.db` is fresh for all files (mtime-based fast path); run indexing lazily in the background.

**Synchronous SQLite audit writes on every tool call:**
- Problem: Each policy decision and execution appends an event to `logs/audit.db` under `Mutex<Connection>` (`audit.rs:27-29`), and each append first does a `SELECT event_hash ... ORDER BY id DESC` (`audit.rs:66-75`) then an INSERT — two round trips per event, serialized.
- Measurement: Not benchmarked; grows linearly with tool calls per session.
- Cause: Hash-chain integrity design with no batching/WAL reuse.
- Improvement path: Batch events in memory and flush periodically; enable WAL; prepare statements once.

**Tool execution is fully serialized:**
- Problem: In `server.rs:334-419` and `main.rs`, tool calls in a round are executed one-by-one in a loop (`for tc in tool_calls`), each with a 30s hard timeout (`tool_runtime.rs:86`).
- Measurement: Multi-tool rounds take sum-of-latencies; web SSE waits.
- Cause: Sequential design for audit ordering simplicity.
- Improvement path: Execute independent tool calls concurrently (bounded) and preserve audit ordering by sequencing inserts.

## Fragile Areas

**`main.rs` / `tui.rs` monoliths:**
- Files: `agent-rs/src/main.rs` (2208 lines), `agent-rs/src/tui.rs` (2077 lines)
- Why fragile: Inline agent loop, retry/OOM heuristics (`main.rs:200-600`), 4 duplicated `gsd-sdk` spawn blocks, TUI state machine with `expect()`s at raw-mode transitions (`tui.rs:1430-1469, 1666-1669`).
- Common failures: A change to streaming logic breaks both terminal and web paths; terminal init failure panics mid-start.
- Safe modification: Extract one concern at a time with tests; keep the SSE parsing (`stream.rs`) and token accounting (`tokens.rs`) untouched as they are unit-tested.
- Test coverage: `stream.rs`, `tokens.rs`, `policy.rs`, `tool_runtime.rs`, `memory.rs`, `sandbox.rs` have unit tests; `main.rs` and `tui.rs` logic are covered only via the integration tests in `agent-rs/tests/*.rs` (23 files, mostly eval scenarios) — no UI tests.

**`scripts/config.py` contract:**
- Files: `scripts/config.py`, `agent-rs/src/config.rs`, `start.py`, `scripts/start_server.py`
- Why fragile: Four consumers depend on exact attribute names (`scan_models_directory`, `build_model_entry`, `DETECTED_VRAM_GB`, `AUDIT_ENABLED`, ...). The Rust bridge fails silently (prints `{'error': ...}` and returns exit 0, `config.rs:101-102`) making breakage hard to diagnose.
- Common failures: Renaming/removing a config symbol breaks the agent with a JSON parse error.
- Safe modification: Add a schema test (`tests/test_config_contract.py`) asserting the attributes the bridge and launchers need.
- Test coverage: `tests/test_qwen_config.py` and `tests/test_model_discovery.py` exercise some of it; no contract test.

**Docker sandbox integration:**
- Files: `agent-rs/src/security/sandbox.rs`, `agent-rs/src/agent_core/tool_runtime.rs:289-351`
- Why fragile: Hard runtime dependency on the local Docker daemon and an unpinned image (`alpine:3.20`, `tool_runtime.rs:313`); tests only validate container config construction (`sandbox.rs:239-278`), never run a container.
- Common failures: Docker absent/daemon down → every terminal tool fails; image not pulled (offline) → failure; nobody-user (`65534:65534`) cannot write host-mounted dirs → unexpected permission errors.
- Safe modification: Add an image pull check + clear error message; document the Docker requirement in README (currently absent).
- Test coverage: No test executes a real container; sandbox behavior in CI is untested.

**`gsd-sdk` external binary contract:**
- Files: `agent-rs/src/main.rs:982, 1039, 1094, 1117`; `apply_fix.py`
- Why fragile: `gsd-sdk` is not declared in `Cargo.toml`, not versioned, and not installed by `setup.py`; the agent spawns it raw from PATH and only surface-level error handling exists ("Failed to run gsd-sdk").
- Common failures: Missing binary → slash commands silently produce an error message; CLI argument drift between agent and SDK.
- Safe modification: Centralize invocation in one helper with `--help` version probing; document install steps; add a test with a stub binary.
- Test coverage: None.

**`setup.py` build chain:**
- Files: `setup.py` (1320 lines)
- Why fragile: Multi-step cascade — rustup install, VS Build Tools via winget, cmake, llama.cpp clone+build with backend fallback (`setup.py:493-543`), token-speed benchmark gate, agentic eval preflight.
- Common failures: Windows vcvars discovery, winget absence, CUDA install failures, backend fallback misdetection, benchmark hang (see Known Bugs).
- Safe modification: Run each stage with clear failure messages (most already do); add `--offline-check` coverage (exists at `setup.py:1095-1124`).
- Test coverage: `tests/test_system_check.py` (1251 bytes) only; the setup flow itself has no automated test.

## Scaling Limits

**Single-process web agent server:**
- Current capacity: One axum process, no connection limits, no rate limiting, permissive CORS (`server.rs`).
- Limit: Concurrency beyond ~dozens of SSE streams degrades; a browser tab or malicious script can open many `/chat` streams (each spawns a tokio task with its own agent loop).
- Symptoms at limit: Memory growth from per-request message buffers; tool execution contention on the SQLite audit mutex.
- Scaling path: Add per-origin/global rate limits (governor), cap concurrent streams, and serialize expensive tool ops.

**SQLite local stores:**
- Current capacity: `logs/audit.db` (audit), `.helix/helix_context.db` (memory + index) — single-user local files.
- Limit: Audit table grows without retention policy; FTS5 index grows with workspace size.
- Symptoms at limit: Startup index time increases; audit query (`--audit-query`) slows.
- Scaling path: Add audit retention/rotation; keep the index incremental (already partially cached).

## Dependencies at Risk

**`gsd-sdk` (undeclared external binary):**
- Risk: Not versioned, not declared in `Cargo.toml`, not installed by setup; contract is implicit.
- Impact: All `/gsd*` slash commands in the TUI degrade to error messages.
- Migration plan: Declare it as a managed dependency (document install, pin version, probe at startup).

**`llama.cpp` (HEAD clone at setup):**
- Risk: `setup.py:503` clones upstream `main` on every fresh setup; no pin.
- Impact: Build breakage when upstream changes; non-reproducible setups.
- Migration plan: Pin a commit SHA (and record it in `scripts/config.py`); consider a submodule.

**`koboldcpp` fallback binary (latest release):**
- Risk: Downloaded from `latest` GitHub release (`setup.py:41-45`), unpinned.
- Impact: Behavior/flag changes break `start_server.py` fallback path.
- Migration plan: Pin a release tag.

**`rustyline` / `tiktoken-rs` / `bollard` / `tachyonfx` (newer versions):**
- Risk: `tiktoken-rs = "0.9.1"`, `bollard = "0.20.2"`, `tachyonfx = "0.11.1"`, edition 2024 (`agent-rs/Cargo.toml`) are recent; API churn risk.
- Impact: Upstream breaking changes require code fixes; toolchain must be new enough for edition 2024.
- Migration plan: Keep `rust-toolchain.toml` pinned (absent today — add one).

## Missing Critical Features

**Web-server authentication / CSRF protection:**
- Problem: The web agent API has no auth or origin validation (see Security).
- Current workaround: Bind to 127.0.0.1 only.
- Blocks: Safe browser-based use of the agent API.
- Implementation complexity: Low (token handshake between `start.py` and the UI).

**Terminal-command execution without Docker:**
- Problem: `run_terminal_command` hard-requires Docker; users without Docker cannot execute any terminal tools.
- Current workaround: None.
- Blocks: Build/test/dev workflows for the primary use case.
- Implementation complexity: Medium (host execution path exists in `tools.rs:491` — wire it behind the policy engine with the existing approval flow).

**Agent API tests:**
- Problem: No tests for `server.rs` web endpoints or the SSE protocol.
- Current workaround: Manual testing via `start.py` web mode.
- Blocks: Confident refactors of the web path.
- Implementation complexity: Low (axum has `tower::ServiceExt::oneshot` testing patterns; mock the LLM upstream with `mockito`, already a dev-dependency).

## Test Coverage Gaps

**`agent-rs/src/server.rs` (web mode):**
- What's not tested: `/chat` SSE handler, compaction loop, tool lifecycle forwarding, `CorsLayer::permissive` behavior.
- Risk: Web mode regressions (port, SSE framing, loop exit) go unnoticed; the 20-round safety exit and compaction code is complex.
- Priority: High
- Difficulty to test: Needs an HTTP harness and a mocked upstream `/v1/chat/completions` (`mockito` is present).

**`web-ui/` (entire frontend):**
- What's not tested: SSE parsing in `App.tsx:54-87` (chunk-split JSON), message/event rendering, error states.
- Risk: Chunk-boundary JSON parse errors silently drop streamed content.
- Priority: Medium
- Difficulty to test: No test framework configured; add Vitest + React Testing Library.

**`start.py` / `setup.py` orchestration:**
- What's not tested: Port conflict fallback (`start.py:147-166`), orphan-process cleanup, model selection flows, config regeneration.
- Risk: The `scripts/config.py` overwrite bug (Tech Debt #1) shipped because nothing exercises setup→start.
- Priority: High
- Difficulty to test: Integration-style; at minimum assert config contract after `generate_config`.

**Docker sandbox execution:**
- What's not tested: Real container runs, image pull failure, workspace mount permissions as `nobody`.
- Risk: The `alpine:3.20` binary-missing bug shipped undetected.
- Priority: Medium
- Difficulty to test: Requires Docker in CI.

**Repair subsystem (`agent_core/repair/`):**
- What's not tested: Snapshot/rollback against real directories on Windows; `snapshots.rs` has a single `panic!` (`snapshots.rs:5` area) path risk; `scoring.rs` is nearly untested (947 bytes).
- Risk: Repair tools could corrupt files without detection.
- Priority: Medium
- Difficulty to test: Platform-dependent (`vssadmin`/`rsync`).

---

*Concerns audit: 2026-08-03*
*Update as issues are fixed or new ones discovered*
