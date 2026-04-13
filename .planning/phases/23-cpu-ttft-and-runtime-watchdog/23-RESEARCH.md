# Phase 23: CPU TTFT and Runtime Watchdog - Research

**Researched:** 2026-04-13
**Domain:** CPU-first runtime tuning, TTFT reduction, server watchdog and controlled recovery
**Confidence:** HIGH (repo integration points), MEDIUM (TTFT threshold calibration per hardware tier)

## Summary

Phase 23 should build on the existing recovery foundation instead of adding a parallel runtime path.

The repo already contains core primitives:
- CPU-safe forced recovery in `agent-rs/src/main.rs` (`maybe_boot_model_server`, `send_with_recovery`).
- Hardware-aware config generation in `scripts/system_check.py` and `setup.py` (threads, batch, ubatch, backend hints).
- Server fallback behavior in `scripts/start_server.py` (primary backend, VRAM/OOM detection, fallback backend/KoboldCPP).

What is still missing for Phase 23:
- A dedicated CPU runtime profile selector focused on TTFT targets (PERF-04).
- A persistent watchdog loop with explicit health states, restart budget, and backoff policy for long-running stability (PERF-05).

Primary recommendation:
1. Add an explicit runtime profile contract (latency-balanced, throughput-balanced, safe) that derives from detected hardware and current backend state.
2. Enforce TTFT-aware profile adaptation at startup and after recovery events.
3. Add a watchdog state machine in Rust orchestrator that monitors health endpoint + chat probe + restart counts with bounded retries and cooldown.
4. Persist watchdog events to logs with clear reason codes so failures are diagnosable and non-looping.

## Standard Stack

### Core (Use)
| Library / Module | Purpose | Why |
|---|---|---|
| Existing Rust `tokio` timing primitives | Health polling, backoff, cooldown scheduling | Already used in runtime retry/recovery logic |
| Existing Rust `reqwest` client | `/v1/models` and lightweight chat readiness probes | Already used by `is_model_server_reachable` and `probe_model_chat_ready` |
| Existing Rust `sysinfo` dependency | Runtime memory/load checks for watchdog thresholds | Already present in `agent-rs/Cargo.toml` |
| Existing Python `scripts/system_check.py` | Hardware tier and backend recommendation | Already computes tier, threads, batch, ubatch, backend hint |
| Existing Python `scripts/start_server.py` | Backend-specific launch + fallback process control | Already central launcher for llama.cpp and KoboldCPP fallback |

### Optional (Only if needed)
| Library | Use case |
|---|---|
| Python `psutil` | Extra process telemetry in watchdog debug mode only |
| Rust `tracing` | Structured watchdog event logs if Phase 23 includes richer observability |

## Architecture Patterns

### Pattern 1: Runtime Profile Contract (PERF-04)

**What:** Introduce explicit runtime profiles with deterministic parameter sets.

Recommended profile keys:
- `profile_name`: `latency_cpu`, `balanced_cpu`, `safe_recovery`
- `backend_hint`
- `gpu_layers`
- `cpu_threads`
- `batch_size`
- `ubatch_size`
- `context_size`
- `ttft_target_ms`

**Why:** Current config generation computes good defaults, but Phase 23 needs an explicit profile object to switch behavior predictably when TTFT degrades.

### Pattern 2: TTFT-First Startup Tuning (PERF-04)

**What:** Prioritize first-token latency over raw throughput on modest CPU hosts.

Recommended startup flow:
1. Load hardware recommendations from existing setup-generated config.
2. Apply CPU TTFT guardrails (cap context and batch for low RAM tiers).
3. Start server with profile params.
4. Run a 1-token chat probe and record TTFT.
5. If TTFT exceeds threshold, downgrade to safer profile and retry once.

**Why:** Avoids perceived freeze while still allowing automatic escalation on stronger systems.

### Pattern 3: Watchdog State Machine (PERF-05)

**What:** Add explicit health states instead of ad hoc retries.

Recommended states:
- `Healthy`
- `Degraded` (probe failures below restart threshold)
- `Recovering` (restart in progress)
- `Cooldown` (too many restarts recently)
- `Unhealthy` (manual intervention required)

Transition signals:
- `/v1/models` health check failures
- chat probe failures/timeouts
- repeated transient HTTP failures from request pipeline
- restart budget exhaustion

**Why:** Prevents infinite restart loops and makes recovery behavior deterministic.

### Pattern 4: Bounded Restart Budget + Backoff (PERF-05)

**What:** Define restart limits and cooldown windows.

Recommended policy:
- Max restarts in window: e.g. 3 restarts / 10 minutes
- Exponential backoff: 1s, 2s, 4s, 8s up to cap
- Cooldown lockout after budget exhaustion
- Clear user/system message with next action when lockout is active

**Why:** Long-running stability depends on bounded recovery, not unlimited retries.

### Pattern 5: Single Recovery Entry Point

**What:** Keep recovery orchestration centralized in Rust runtime and delegate process launch to existing Python launcher.

Recommendation:
- Keep `send_with_recovery` as the top-level request-path recovery gate.
- Route all server restarts through one helper that applies profile overrides via env vars (`HELIX_BACKEND_HINT`, `HELIX_GPU_LAYERS`, etc.).
- Do not add separate watchdog launch scripts.

**Why:** Existing architecture already uses this split; duplicating recovery paths will drift quickly.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Runtime hardware scoring | new custom scorer in Rust | existing `scripts/system_check.py` outputs and config | Hardware logic is already implemented and battle-tested in setup flow |
| Health checks | complex bespoke protocol | existing `/v1/models` + minimal chat probe | Current runtime already uses these checks and they are sufficient |
| Recovery launch stack | new process launcher | existing `scripts/start_server.py` with env overrides | Prevents duplicate fallback logic |
| Retry policy | unbounded while-true loops | bounded retry + backoff + cooldown state machine | Required for stability and operator trust |
| TTFT measurement | manual wall-clock prints scattered in code | centralized TTFT probe helper with threshold comparison | Keeps adaptation logic consistent and testable |

## Common Pitfalls

### 1) Optimizing for throughput when user pain is TTFT
Large batch/context settings can improve tokens/sec but worsen first-token latency on CPU-only systems.

### 2) Silent fallback changes
If backend/profile changes happen without explicit status messages, users perceive random behavior. Always emit mode/profile transition messages.

### 3) Infinite recovery loops
Repeated automatic restarts without budget limits can hide root causes and degrade system availability.

### 4) Splitting watchdog logic across Python and Rust
Keep orchestration ownership in one layer (Rust runtime), with Python launcher as execution backend.

### 5) Treating health endpoint success as full readiness
`/v1/models` can be up while chat generation is still unhealthy. Keep chat probe in readiness checks.

## Code Examples

### Example 1: Runtime profile object

```rust
struct RuntimeProfile {
    name: String,
    backend_hint: String,
    gpu_layers: i32,
    cpu_threads: usize,
    batch_size: usize,
    ubatch_size: usize,
    context_size: usize,
    ttft_target_ms: u64,
}
```

### Example 2: TTFT probe with one fallback profile downgrade

```rust
async fn ensure_ttft_budget(client: &reqwest::Client, cfg: &AppConfig, profile: &RuntimeProfile) -> bool {
    let ttft_ok = probe_ttft_ms(client, cfg).await <= profile.ttft_target_ms;
    if ttft_ok {
        return true;
    }

    let safer = RuntimeProfile {
        name: "safe_recovery".to_string(),
        batch_size: profile.batch_size.min(256),
        context_size: profile.context_size.min(2048),
        ..profile.clone()
    };

    apply_profile_env(&safer);
    restart_server_once().await && probe_ttft_ms(client, cfg).await <= safer.ttft_target_ms
}
```

### Example 3: Watchdog state transition skeleton

```rust
enum WatchdogState {
    Healthy,
    Degraded,
    Recovering,
    Cooldown,
    Unhealthy,
}

fn transition(state: WatchdogState, health_ok: bool, restart_budget_ok: bool) -> WatchdogState {
    match (state, health_ok, restart_budget_ok) {
        (WatchdogState::Healthy, false, true) => WatchdogState::Degraded,
        (WatchdogState::Degraded, false, true) => WatchdogState::Recovering,
        (WatchdogState::Recovering, false, false) => WatchdogState::Cooldown,
        (WatchdogState::Cooldown, false, false) => WatchdogState::Unhealthy,
        (_, true, _) => WatchdogState::Healthy,
        (s, _, _) => s,
    }
}
```

### Example 4: Controlled restart budget check

```rust
fn can_restart(restarts_in_window: usize, max_restarts: usize) -> bool {
    restarts_in_window < max_restarts
}
```

## Source Evidence

- `agent-rs/src/main.rs` already has transient HTTP classification, server reachability probes, and auto-boot recovery (`send_with_recovery`, `maybe_boot_model_server`).
- `scripts/start_server.py` already applies runtime overrides, starts llama.cpp, detects OOM-like failures from logs, and attempts fallback backend/KoboldCPP.
- `scripts/system_check.py` already computes backend recommendations and tier-based `threads`, `batch_size`, `ubatch_size`, `context_size`, and `gpu_layers`.
- `setup.py` already writes these tuning values to runtime config and enforces a basic performance gate before finishing setup.

## Implementation Readiness

Ready for `/gsd-plan-phase 23`.

Recommended planning order:
1. Define runtime profile contracts and watchdog state model (types and config boundaries).
2. Implement TTFT-aware profile selection and adaptation flow at startup (PERF-04).
3. Implement watchdog loop with bounded restart policy and cooldown semantics (PERF-05).
4. Add regression tests for profile downgrade behavior, restart budget enforcement, and health-probe-driven state transitions.
