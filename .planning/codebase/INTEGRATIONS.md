# External Integrations

**Analysis Date:** 2025-05-15

## APIs & External Services

**Local Inference:**
- llama.cpp - Primary inference engine for Qwen 3.6 models.
  - SDK/Client: `async-openai` in Rust.
  - Auth: Localhost only, no auth required.

**Model Repositories:**
- HuggingFace - Source for Qwen 3.6 GGUF weights.
  - Scripted download via `scripts/model_install.py`.

## Data Storage

**Databases:**
- SQLite (Local)
  - Connection: `rusqlite` in `agent-rs`.
  - Client: Direct `rusqlite` usage for audit logs (`agent-rs/src/audit.rs`) and session persistence (`agent-rs/src/session.rs`).

**File Storage:**
- Local filesystem only. Workspace managed in `agent-rs/src/security/policy.rs`.

**Caching:**
- None (Local model weights are persistent).

## Authentication & Identity

**Auth Provider:**
- Custom
  - Implementation: Permission tiers (ReadOnly, WorkspaceWrite, FullExec) in `agent-rs/src/security/policy.rs`.

## Monitoring & Observability

**Error Tracking:**
- Local Logs
  - Location: `logs/` and `agent-rs/logs/`.

**Logs:**
- Structured Audit Store
  - Implementation: `agent-rs/src/audit.rs` captures tool calls, policy decisions, and execution outcomes.

## CI/CD & Deployment

**Hosting:**
- Local Workstation (Privacy-first).

**CI Pipeline:**
- GitHub Actions (for llama.cpp and agent-rs builds).

## Environment Configuration

**Required env vars:**
- `HELIX_RUNTIME_PROFILE` - Selects latency vs. accuracy profile.
- `HELIX_BACKEND_HINT` - Hints `llama-server` vs. `koboldcpp`.
- `HELIX_GPU_LAYERS` - Manual VRAM offload control.

**Secrets location:**
- Not applicable (Local-first, secrets stay in OS keychain or environment).

## Webhooks & Callbacks

**Incoming:**
- `/v1/chat/completions` - OpenAI-compatible endpoint exposed by `agent-rs/src/server.rs`.

**Outgoing:**
- None (Local execution only).

---

*Integration audit: 2025-05-15*
