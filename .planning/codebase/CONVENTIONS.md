# Coding Conventions

**Analysis Date:** 2026-08-03

Multi-language repo: **Python** (launcher/setup/CLI + `scripts/` modules + `tests/`), **Rust** (`agent-rs/` agent), **TypeScript/React** (`web-ui/`, no tests). Python is the dominant convention surface. `llama.cpp/` is vendored C++ (excluded from these conventions).

## Naming Patterns

**Files:**
- Python modules/scripts: `snake_case.py` (`model_install.py`, `start_server.py`, `download_model.py`)
- Python tests: `test_<module>.py` in `tests/` (`test_model_install.py`, `test_qwen_config.py`)
- Rust source: `snake_case.rs` per module, `mod.rs` for module roots (`agent-rs/src/agent_core/mod.rs`, `src/security/mod.rs`)
- Rust integration tests: `snake_case.rs` in `agent-rs/tests/` (`tool_execution.rs`, `audit_log_mvp.rs`)
- Web: `App.tsx`, `main.tsx` in `web-ui/src/`; configs `vite.config.ts`, `tailwind.config.js`

**Functions:**
- Python: `snake_case` (`verify_model_integrity`, `build_model_entry`, `scan_models_directory`)
- Private helpers: underscore prefix (`_safe_int`, `_normalise_command`, `_parse_parameter_count`, `_home_dir`, `_cuda_candidate_gpu_layers`)
- CLI entry: `main(argv=None) -> int` + `if __name__ == "__main__":` guard
- Rust: `snake_case` (`strip_reasoning_blocks`, `deduplicate_consecutive_sentences`, `clean_chat_output`)

**Variables:**
- Python: `snake_case` (`model_path`, `staged_path`, `gpu_layers`)
- Python module-level constants: `UPPER_SNAKE_CASE` (`TRUSTED_MODELS`, `MODELS_DIR`, `BLOCKLIST`, `TIER_CONFIGS`, `DEFAULT_MODELS`, `KOBOLD_URLS`)
- Rust: `snake_case` locals; private items have no prefix

**Types:**
- Python: hints from `typing` (`Dict[str, Any]`, `Optional[Path]`, `List[ModelEntry]`); dataclasses for data structs — `@dataclass(frozen=True) class ModelEntry` in `scripts/config.py:13`
- Rust: `CamelCase` structs/enums (`AppConfig`, `BackendCapabilities`, `SseEvent`, `SseParser`), derive `Debug, Clone, Serialize, Deserialize`

## Code Style

**Formatting:**
- No formatter config detected (no black/isort/ruff config anywhere in repo)
- 4-space indentation, double quotes for Python strings
- Rust: rustfmt defaults (no `rustfmt.toml`); 2024 edition (`agent-rs/Cargo.toml`)
- Web: ESLint flat config only; no semicolons in `web-ui/eslint.config.js`

**Linting:**
- No Python linter config; `# noqa: F401` used inline in `setup.py:99`
- Rust: no clippy config
- Web: `web-ui/eslint.config.js` — extends `@eslint/js` recommended + `typescript-eslint` recommended + `react-hooks` + `react-refresh`; run `npm run lint` in `web-ui/`

## Import Organization

**Order:**
1. Standard library (`os`, `sys`, `subprocess`, `pathlib`, `typing`)
2. Third-party (`requests`, `pytest`, `tqdm`, `huggingface_hub`)
3. Local modules (`from model_install import install_model`, `import config`, `import system_check`)

**Grouping:**
- Blank line between groups; stdlib alphabetical within group
- Local imports after the sys.path bootstrap block
- Rust: `use` at top; crate-internal via `use crate::...` (`use crate::security::policy::PermissionTier` in `agent-rs/src/config.rs:5`)

**Path Aliases / Import mechanics:**
- No `__init__.py` — `scripts/` is a flat importable directory
- Scripts and tests bootstrap `sys.path` (see `tests/test_model_install.py:9`, `scripts/system_check.py:15`):
```python
PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))
```
- Root scripts import via package path: `from scripts.helix_branding import print_helix_logo` (`start.py:9`)
- Rust tests import either `use agent_rs::...` (lib crate) or `#[path = "../src/stream.rs"] mod stream;` for binary-private modules (`agent-rs/tests/streaming_tui_refinement.rs:1`)

## Error Handling

**Patterns:**
- Entry points / CLI: print `[!]` message then `sys.exit(1)` — fail fast at boundaries (`setup.py`, `start_server.py`, `helix.py`)
- Library functions: raise `ValueError` for invalid input — `validate_trusted_model_spec()` (`scripts/model_install.py:77`), `list_repo_files()` (`scripts/download_model.py:135`)
- Expected failures return falsy/Optional instead of raising: `resolve_model_ref()` → `None`, `install_model_spec()` → `bool`, `verify_model_integrity()` → `bool`
- Broad `except Exception` at boundaries with graceful degradation (e.g. `install_model_spec` catches and returns `False`; `setup.py:142` handles rust install failure with message + `sys.exit(1)`)
- `try/finally` guarantees teardown: `stop_process(proc)` + close file handles (`setup.py:763`, `start.py:374`)
- Guard clauses + early returns (`if not path.exists(): return False`)

**Error Types:**
- Throw when: invalid input, missing required metadata (no SHA256, no `.gguf` files, unpinned revision)
- Return when: expected failure (file missing, unknown model ref, backend unavailable)
- Rust: errors as `String` via `Result<T, String>` (`AppConfig::load_from_python() -> Result<Self, String>` in `agent-rs/src/config.rs:43`); `.unwrap()`/`.expect()` used in tests and internal code

## Logging

**Framework:** None — plain `print()`. No `logging` module, no third-party logger.

**Patterns:**
- Status prefixes: `[!]` error/warning, `[✓]` success, `[i]` info, plus contextual `[Runtime]`, `[Model Discovery]`, `[Warning]`
- Two-space indent for detail lines under a status line
- Banner separators: `"-" * 55` and `"=" * 55`
- Print the command before executing: `print(f"  $ {quote_cmd(cmd)}")` (`setup.py:55`)
- Emoji markers in benchmark scripts (`tests/eval.py` uses ✅ ⚠️ 🔎)

## Comments

**When to Comment:**
- Explain "why" and non-obvious tradeoffs: `# Do not hard-block setup on non-admin shells. Most steps (downloads/builds) work fine unprivileged...` (`setup.py:89`)
- Document tuning rationale: `# Wider sweep for 4GB cards to expose real peak capability across offload profiles.` (`setup.py:781`)
- Section banners: `# ─── Tier → Config Mapping (targeting ≥10 tok/s) ───` (`scripts/system_check.py:21`)
- No boilerplate comments on obvious code

**Docstrings:**
- Module-level docstrings on scripts (`setup.py:2`, `build_zip.py:2`, `helix_branding.py:2`)
- Function docstrings on public/API functions: `detect_cpu()` (`scripts/system_check.py:75`), `apply_runtime_overrides()` (`scripts/start_server.py:28`)
- Test docstrings describe the behavior under test
- Rust: `///` on public items (`agent-rs/src/lib.rs:21`), `//!` module docs in integration tests (`agent-rs/tests/context_integration.rs:1`)

**TODO Comments:**
- Only one in repo: `// TODO: extract doc comments in a follow-up` in `agent-rs/src/context/indexer.rs:330` (no username/issue link)

## Function Design

**Size:**
- No strict limit; `setup.py` `main()` is ~240 lines. Heavy extraction into helpers is the norm (`_safe_int`, `_measure_token_speed_once`, `_cuda_candidate_gpu_layers`)

**Parameters:**
- Positional params with defaults; call sites with 6+ args pass keywords — `llm_cmd(...)` (`setup.py:699`), `generate_config(...)` (`setup.py:1245`), `install_model_spec(...)` calls
- Optional values typed `Optional[...] = None`

**Return Values:**
- Explicit returns everywhere; guard clauses return early
- Success/failure checks return `bool`; lookups return `Optional[X]`; CLI `main()` returns `int` exit code
- Rust functions return `String`/`Vec`/`Result` with explicit paths

## Module Design

**Exports:**
- Python CLI modules export `main(argv=None) -> int` then guard at bottom: `if __name__ == "__main__": sys.exit(main())` (`scripts/model_install.py:219`) or `main()` (`setup.py:1319`)
- `scripts/config.py` is a computed-constants module (module-level env override block, `scripts/config.py:402`)
- Rust: `lib.rs` declares `pub mod` for every module and `pub use` re-exports key types (`agent-rs/src/lib.rs:19`)
- Rust submodule barrels: `mod.rs` in `agent_core/`, `context/`, `security/`, `tui/`, `web_research/`

**Barrel Files:**
- Rust `mod.rs` used as module barrels; no Python barrel/`__init__.py` pattern

---

*Convention analysis: 2026-08-03*
*Update when patterns change*
