# External Integrations

**Analysis Date:** 2026-08-03

## APIs & External Services

**Model Registry (HuggingFace Hub):**
- HuggingFace - GGUF model download and repo search during setup (`setup.py`, `scripts/model_install.py`, `scripts/download_model.py`)
  - SDK/Client: `huggingface_hub` (`hf_hub_download`, `HfApi`) with `requests` + `tqdm` streaming fallback
  - Auth: none (public repos; no HF token required)
  - Endpoints used: `https://huggingface.co/api/models` (search), `https://huggingface.co/api/models/{repo}/tree/main` (file listing), `https://huggingface.co/{repo}/resolve/{revision}/{filename}` (download)
  - Integrity: pinned git revisions + SHA256 verification enforced in `TRUSTED_MODELS` (`scripts/model_install.py`); models flagged `UNVERIFIED_REVISION` are blocked until pinned
  - Known repos: `Qwen/Qwen3.6-27B-Instruct-GGUF`, `Qwen/Qwen3.6-35B-Instruct-GGUF`, `DavidAU/OpenAi-GPT-oss-20b-abliterated-uncensored-NEO-Imatrix-gguf`, `HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive`, `unsloth/gemma-4-E4B-it-GGUF`

**Source/Binary Downloads:**
- KoboldCPP - Fallback inference binary (only used when llama.cpp fails)
  - Integration method: direct download from `https://github.com/LostRuins/koboldcpp/releases/latest/download/` (per-OS filename: `koboldcpp.exe`, `koboldcpp-linux-x64`, `koboldcpp-mac-x64`) (`setup.py` `KOBOLD_URLS`)
  - Auth: none
- llama.cpp - Inference engine source, cloned from `https://github.com/ggerganov/llama.cpp.git` when missing (`setup.py` `build_llama_cpp`); not a git submodule, build output gitignored

**LLM Inference API (local, primary runtime integration):**
- llama-server (llama.cpp) or KoboldCPP - OpenAI-compatible `/v1` HTTP server on `127.0.0.1:8080` (port overridable via `HELIX_SERVER_PORT`)
  - Integration method: OpenAI-compatible REST API consumed by the Rust agent via `async-openai 0.33` and `reqwest` (`agent-rs/src/main.rs`, `agent-rs/src/server.rs`)
  - Endpoints used: `GET /v1/models`, `POST /v1/chat/completions` (streaming), `POST /v1/completions` (benchmark), `GET /props` (capability probe: `has_jinja` → function calling)
  - Capability detection: `probe_backend_capabilities` in `agent-rs/src/config.rs` (function calling enabled for gemma/functionary/hermes model IDs or `has_jinja`)
  - Backend flavor detection (`ServerFlavor::LlamaCpp`/`KoboldCpp`) in `agent-rs/src/main.rs:19` — KoboldCpp gets GBNF grammar tool calling instead of native function calls
  - Server process managed by `scripts/start_server.py` (llama-server → VRAM-OOM fallback → KoboldCPP chain)

**Web Research (agent tool):**
- DuckDuckGo HTML search - `https://html.duckduckgo.com/html/?q=...` (no API key, HTML scraping) (`agent-rs/src/agent_core/web_research/planner.rs`)
- crates.io and docs.rs - Rust crate documentation lookups (`https://crates.io/crates/{name}`, `https://docs.rs/{name}`) (`agent-rs/src/agent_core/web_research/planner.rs`)
- Arbitrary URLs - fetched directly with `reqwest`, sanitized to markdown via `scraper` + `htmd` (`agent-rs/src/agent_core/web_research/sanitize.rs`)
  - Security: SSRF guard rejects loopback/private ranges (e.g. `127.0.0.1`, `10.0.0.0/8`, `169.254.169.254`) before fetching

**System/GPGPU Detection:**
- nvidia-smi - GPU VRAM detection (`nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits`) (`scripts/config.py` `detect_gpu_vram_gb`); GPU stats also read by the Rust agent
- Docker daemon - sandboxed command execution via local socket (`Docker::connect_with_local_defaults()` in `agent-rs/src/security/sandbox.rs`, bollard SDK; containers named `helix-sandbox-{ts}`)

## Data Storage

**Databases:**
- SQLite (embedded) - Command audit log
  - Client: `rusqlite 0.39` with `bundled` feature (no external SQLite dependency) (`agent-rs/src/audit.rs`)
  - Location: `logs/audit.db` (path from `AUDIT_DB_PATH` in `scripts/config.py`); `agent-rs/test_audit*.db` are gitignored cargo-test artifacts

**File Storage:**
- Local filesystem only - no cloud storage
  - Models: `models/` (GGUF files, gitignored; `.staging/` used for verified downloads)
  - Logs: `logs/` (server stdout/stderr, audit DB)
  - User state: `~/.helix/onboarding_profile.json` + `~/.helix/sessions/` (JSON files, `scripts/onboarding_profile.py`)

**Caching:**
- None (no Redis/memcached; context/memory handled in-process by `agent-rs/src/context/memory.rs`)

## Authentication & Identity

**Auth Provider:**
- None - fully local, no user accounts or auth. The only "identity" is the local user profile at `~/.helix/onboarding_profile.json` (plain JSON, no credentials)

**OAuth Integrations:**
- None

**API Keys/Secrets:**
- None stored or required (all external services are public-download or local-loopback). No `.env` files exist in the repo.

## Monitoring & Observability

**Error Tracking:**
- None (no Sentry/other)

**Analytics:**
- None

**Logs:**
- Local file logs only: `logs/start_server.stdout.log`, `logs/start_server.stderr.log`, per-run `logs/{tag}_{timestamp}.stdout.log`/`.stderr.log` written by `setup.py`/`start.py`
- Command audit trail in SQLite (`logs/audit.db`) with tamper-evident hashes (`agent-rs/src/audit.rs`)

## CI/CD & Deployment

**Hosting:**
- None - distributed as local source/zip (`scripts/build_zip.py`); no deployment target

**CI Pipeline:**
- None - no `.github/` directory; validation performed by `tests/eval.py` benchmark + pytest + cargo test run locally, plus `python setup.py --offline-check`

## Environment Configuration

**Development:**
- No env files; `HELIX_*` variables set ad-hoc or exported in shell. Full list in `STACK.md` under Configuration
- Setup-time gates: `HELIX_MIN_TOK_S` (token speed threshold, default 10 tok/s), `HELIX_RUN_AGENTIC_PREFLIGHT`, `HELIX_EVAL_MAX_TASKS` (default 4), `HELIX_EVAL_CATEGORIES`
- Secrets location: not applicable (no secrets)

**Production:**
- N/A (local-first; no server environments)

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None (the agent's HTTP calls are request/response; no event-driven callbacks registered)

---

*Integration audit: 2026-08-03*
*Update when adding/removing external services*
