# Concerns

## Build/Runtime Fragility
- Mixed platform artifacts in `llama.cpp/build/` can complicate binary detection.
- Windows-specific process locking can block Rust rebuilds (`agent-rs.exe` locked).
- Startup depends on multiple subprocesses; failures can appear as generic timeouts.

## UX Concerns
- Interactive orchestrator previously leaked internal reasoning and round counters.
- Startup prompts changed frequently; user expectations around "normal chatbot" were mismatched.
- Legacy terminal encoding can break unicode-heavy branding output.

## Configuration Drift Risks
- Setup-generated `scripts/config.py` can drift from runtime overrides.
- Benchmark and preflight behavior has changed multiple times, increasing policy complexity.
- Eval assumptions around tool-call output format may drift with orchestrator changes.

## Performance and Resource Risks
- Model startup latency can exceed short timeout windows.
- 8K+ context minimum increases memory pressure on lower-tier systems.
- Fallback behavior may mask root causes unless logs are surfaced clearly.

## Security and Safety
- Terminal execution capability is powerful; dangerous command filtering must remain strict.
- Logging and benchmark traces may include sensitive local path details.
- Secret-scanning should be run before committing generated planning docs.

## Recommended Hardening
- Add stable startup health checks with explicit backend status reporting.
- Keep conservative defaults for preflight and optional heavy benchmarks.
- Introduce CI smoke checks for launcher flow and binary path detection.
