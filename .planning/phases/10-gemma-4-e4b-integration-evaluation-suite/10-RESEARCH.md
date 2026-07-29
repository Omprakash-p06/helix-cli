# Phase 10 Research: Gemma 4 E4B Integration & Evaluation Suite

## RESEARCH COMPLETE

---

## 1. Gemma 4 E4B-it Model

### Key Capabilities
- **Architecture:** Effective 4B parameter model (Gemma 4 family by Google DeepMind)
- **Context Window:** 128K tokens
- **Modalities:** Text, image, and audio (multimodal)
- **Function Calling:** Native support via structured JSON tool call format
- **Instruction Format:** IT (instruction-tuned) variant optimized for agentic tasks
- **Chat Template:** Gemma-style `<start_of_turn>user` / `<end_of_turn>` template, stored in `tokenizer_config.json`

### GGUF Availability
- **Primary HF Repo:** `unsloth/gemma-4-E4B-it-GGUF`
- **Key Quantization Files:**
  - `gemma-4-E4B-it-Q4_K_M.gguf` — balanced, recommended for 4–8GB VRAM
  - `gemma-4-E4B-it-Q8_0.gguf` — high quality, needs 8GB+ VRAM
  - `gemma-4-E4B-it-Q5_K_M.gguf` — medium quality, needs 6GB+ VRAM
  - `mmproj-F16.gguf` — multimodal projector file (required for vision tasks)
- **Download method:** Existing `scripts/download_model.py` HuggingFace downloader (Phase 05)

### Function Calling Format
- **Mechanism:** Gemma 4 emits structured JSON tool call objects during generation
- **Integration via llama.cpp server (`--jinja` flag):** Start llama-server with `--jinja` to apply the model's chat template correctly
- **Tool declaration format:** System-level tool declarations via `<start_function_declaration>` tags (model-native)
- **OpenAI-compatible:** When served via `llama-server`, tool calls appear as standard OpenAI `tool_call` JSON objects — compatible with existing `agent-rs` HTTP client
- **BackendCapabilities.function_calling:** Must be set to `true` after querying the model's `/v1/models` endpoint to verify tool-call capability

### KV Cache / 128K Context
- 128K context requires `--ctx-size 131072` (or `context_size: 131072` in config)
- KV cache memory at 128K ≈ 2–4GB additional VRAM for 4-bit quant
- Recommend chunked-attention or flash-attention (`--flash-attn`) to keep KV cache manageable
- For 8GB VRAM systems: use Q4_K_M with `--ctx-size 32768` (cap at 32K) to avoid OOM
- Existing `AppConfig.context_size` field controls this; must add Gemma 4 variants to `_static_model_catalog()`

---

## 2. llama.cpp Integration

### Support Status
- Gemma 4 GGUF format fully supported in llama.cpp (as of mid-2025)
- No special compilation flags beyond standard CUDA build: `cmake -DGGML_CUDA=ON`
- **Required flag:** `--jinja` for Jinja2 template rendering of the model's chat template (critical for tool calling)
- **Server startup command:**
  ```bash
  ./llama-server \
    --model models/gemma-4-E4B-it-Q4_K_M.gguf \
    --mmproj models/mmproj-F16.gguf \
    --jinja \
    --ctx-size 32768 \
    --host 127.0.0.1 \
    --port 8080 \
    --flash-attn
  ```
- Flash attention (`--flash-attn`) significantly reduces memory for long contexts

### Integration Points in Helix Agent
- `scripts/config.py` `_static_model_catalog()` → add `"Gemma-4-E4B"` entry with VRAM tiers
- `scripts/config.py` `build_model_entry()` → no changes needed (uses existing pattern)
- `scripts/start_server.py` → must add `--jinja` and `--mmproj` flags when model is Gemma 4
- `agent-rs/src/config.rs` `BackendCapabilities` → probe `/v1/models` or `/props` endpoint for `function_calling: true`
- Existing `BackendCapabilities.context_window: u32` field → populate from response or config

---

## 3. Evaluation Suite Design (8 Scenarios)

### Standard Eval Categories for AI Coding Agents
Per the roadmap success metrics, Phase 10 needs 8 evaluation scenarios. Based on industry practice (SWE-bench, LMSYS, HellaSwag for retention):

| # | Scenario | Pass Condition | Test Type |
|---|----------|----------------|-----------|
| 1 | **Repo Comprehension** | Agent correctly identifies the file and line number of a queried symbol from the codebase | Integration test using existing `search_codebase` tool (Phase 08) |
| 2 | **Tool-Call Correctness** | Agent produces well-formed JSON tool calls with correct parameters for ≥95% of prompts | Integration test sending structured prompts, parsing `tool_call` JSON |
| 3 | **Long-Session Retention** | After 3 compaction cycles, agent correctly recalls ≥90% of injected constraints | Integration test against `MemoryEngine` (Phase 08 durable memory) |
| 4 | **Research Factuality** | `WebResearchPipeline` returns citations; synthesized brief contains correct fact | Integration test mocking HTTP fetch, asserting evidence cites source URL |
| 5 | **Prompt-Injection Resistance** | Malicious content inside `<untrusted_web_content>` does not appear in agent's next system prompt | Adversarial test extending `web_research_adversarial.rs` (Phase 09) |
| 6 | **Policy-Escape Resistance** | Agent refuses to execute blocklisted commands even when prompted via role-play or jailbreak text | Security test asserting `is_blocked_command()` returns true and tool returns error |
| 7 | **Rollback Correctness** | After a failed repair, agent correctly restores pre-repair snapshot state | Integration test using Phase 03 `FIX-02` snapshot mechanism |
| 8 | **End-to-End Research → Code** | Full pipeline: query → FreshnessClassifier → WebResearchPipeline → ContextEngine → tool call produces valid JSON response | End-to-end integration test with Gemma 4 model |

### Rust Test Harness Pattern
- **Location:** `agent-rs/tests/eval_suite_*.rs` — one file per scenario group
- **Framework:** Rust's built-in `#[test]` + `tokio::test` for async scenarios
- **Mocking:** `mockito` crate for HTTP mocking (HTTP fetch in WorkerPool); `tempfile` for SQLite test DBs
- **Benchmarking:** `criterion` crate for latency-sensitive tests (3s symbol lookup budget from Phase 08)
- **Reporting:** Custom `EvalResult { scenario: String, passed: bool, duration_ms: u64, notes: String }` struct; serialize to JSON with `serde_json` at end
- **CI integration:** `cargo test --package agent-rs --test eval_suite_ -- --test-threads=1` (serial to avoid DB conflicts)

---

## 4. Integration Architecture

### New Config Entry (scripts/config.py)
```python
"Gemma-4-E4B": {
    "repo_alias": "gemma-4-e4b",
    "variants": [
        {
            "min_vram_gb": 8,
            "quantization": "Q8_0",
            "filename": "gemma-4-E4B-it-Q8_0.gguf",
            "mmproj_filename": "mmproj-F16.gguf",
            "gpu_layers": -1,
            "backend_hint": "cuda",
            "context_size": 32768,
            "batch_size": 512,
            "ubatch_size": 256,
            "jinja": True,
            "guidance": "8GB+ VRAM: run Gemma 4 E4B in Q8_0.",
        },
        {
            "min_vram_gb": 4,
            "quantization": "Q4_K_M",
            "filename": "gemma-4-E4B-it-Q4_K_M.gguf",
            "mmproj_filename": "mmproj-F16.gguf",
            "gpu_layers": -1,
            "backend_hint": "cuda",
            "context_size": 16384,
            "batch_size": 256,
            "ubatch_size": 128,
            "jinja": True,
            "guidance": "4GB+ VRAM: run Gemma 4 E4B in Q4_K_M.",
        },
        {
            "min_vram_gb": 0,
            "quantization": "Q4_K_M",
            "filename": "gemma-4-E4B-it-Q4_K_M.gguf",
            "mmproj_filename": "mmproj-F16.gguf",
            "gpu_layers": 0,
            "backend_hint": "cpu",
            "context_size": 8192,
            "batch_size": 128,
            "ubatch_size": 64,
            "jinja": True,
            "guidance": "CPU fallback: Gemma 4 E4B in Q4_K_M (slow).",
        },
    ],
}
```

### Key Files to Modify
1. `scripts/config.py` — Add Gemma 4 E4B catalog entry + `jinja` and `mmproj_filename` fields
2. `scripts/start_server.py` — Pass `--jinja` and `--mmproj` flags when model has these fields
3. `scripts/download_model.py` — Add Gemma 4 E4B as a downloadable preset (HF repo: `unsloth/gemma-4-E4B-it-GGUF`)
4. `agent-rs/src/config.rs` — `BackendCapabilities` probing on startup; populate `context_window` from model config
5. `agent-rs/tests/eval_suite_*.rs` — 8 evaluation scenario files

---

## Validation Architecture

### Validation Sampling Points
- **After every task commit:** `cargo test --package agent-rs eval_suite_ -q` (subset)
- **Full suite:** `cargo test --package agent-rs -q && pytest tests/`
- **Estimated runtime:** ~30s for Rust tests (no live model required — mock-based); ~5s for pytest

### Test Framework
- **Rust unit/integration:** `cargo test` with `tokio::test`, `mockito`, `tempfile`
- **Python:** `pytest` (already at 47 passing tests)
- **No live model required** for eval_suite — use mock HTTP responses and pre-recorded fixtures
