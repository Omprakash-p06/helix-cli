# Phase 05: Model Management & UX Polish - Research

**Researched:** 2026-04-26
**Domain:** Rust TUI + Python launcher + Hugging Face model distribution
**Confidence:** HIGH

## Summary

The repo already separates the main responsibilities for this phase: Python code handles model discovery, trusted install/download, and runtime env handoff, while the Rust TUI owns interactive input, slash-command UX, and GSD autofill behavior. [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/model_install.py`, `scripts/download_model.py`, `start.py`, `agent-rs/src/tui.rs`, `agent-rs/src/tui/state.rs`, `agent-rs/src/tui/approval.rs`]

The safest implementation path is to keep one source of truth for model selection and refresh it from the local `models/` directory after installs, rather than adding a second catalog or a separate launch path. [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/start_server.py`, `start.py`] For the startup picker, the lightest terminal-native choice is `inquire::Select`; if the selection must stay inside the full-screen app, ratatui's documented stateful widgets are the correct pattern. [CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html; CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html; CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html]

**Primary recommendation:** extend the existing launcher and model-install flow so the user can discover, select, and download models without duplicating model state across Python and Rust. [VERIFIED/CITED sources above]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Local model discovery from `models/` and fallback catalog generation | API / Backend | Database / Storage | The scan, catalog assembly, and fallback logic are application-side decisions over filesystem state, while the actual model files are storage. [VERIFIED: repo code inspection of `scripts/config.py`] |
| Startup model selection menu | Browser / Client | API / Backend | The picker collects user intent in an interactive surface, then hands the choice to runtime via the launcher/server boundary. [VERIFIED: repo code inspection of `start.py` and `scripts/start_server.py`; CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html] |
| GGUF download, staging, and integrity verification | API / Backend | Database / Storage | Downloading, checksum validation, and atomic activation are backend responsibilities; the downloaded artifacts live in storage. [VERIFIED: repo code inspection of `scripts/model_install.py`, `scripts/download_model.py`] |
| GSD slash-command autofill and ghost suggestion | Browser / Client | API / Backend | The suggestion lives in the TUI input layer, but the command it suggests must still resolve through the orchestrator's command registry. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`, `agent-rs/src/tui/commands.rs`, `agent-rs/src/main.rs`] |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 (repo pin: 0.26) | Stateful terminal UI rendering | Latest crate version is 0.30.0, and the repo already uses ratatui for the TUI. [VERIFIED: `cargo search ratatui --limit 1`; VERIFIED: `agent-rs/Cargo.toml`] |
| crossterm | 0.29.0 (repo pin: 0.27) | Terminal event/input backend | Latest crate version is 0.29.0, and the repo already uses crossterm for terminal control. [VERIFIED: `cargo search crossterm --limit 1`; VERIFIED: `agent-rs/Cargo.toml`] |
| tui-input | 0.15.3 (repo pin: 0.8) | Text input handling for the TUI | Latest crate version is 0.15.3, and the repo already uses tui-input for prompt editing. [VERIFIED: `cargo search tui-input --limit 1`; VERIFIED: `agent-rs/Cargo.toml`] |
| inquire | 0.9.4 (repo pin: 0.9) | Terminal selection and confirmation prompts | Latest crate version is 0.9.4, and the repo already uses inquire for terminal prompts. [VERIFIED: `cargo search inquire --limit 1`; VERIFIED: `agent-rs/src/tui/approval.rs`] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| huggingface_hub | 1.12.0 (installed 1.8.0) | Hub metadata, repo-tree listing, and file download helpers | Use for HF GGUF discovery/download flow instead of raw HTTP tree scraping. [VERIFIED: `python3 -m pip index versions huggingface_hub`; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api] |
| requests | 2.33.1 (installed 2.32.5) | HTTP client for Python-side scripts that still use direct requests | Use only where the phase intentionally keeps a lightweight HTTP path. [VERIFIED: `python3 -m pip index versions requests`] |
| tqdm | 4.67.3 | Progress bars for long-running downloads | Use for file-transfer progress rather than a custom terminal progress renderer. [VERIFIED: `python3 -m pip index versions tqdm`; CITED: https://tqdm.github.io/] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| terminal prompt selection | in-TUI stateful list/table picker | Prompt selection is simpler to launch; in-TUI selection keeps the user inside the app but needs more state management. [CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html; CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html] |
| raw `requests.get` tree browsing | `huggingface_hub` repo-tree and download helpers | The Hub client handles repository metadata, auth-aware access, and download semantics instead of hand-rolled URL handling. [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download] |
| custom progress reporting | `tqdm` | `tqdm` already solves the long-running download UX without extra terminal rendering logic. [CITED: https://tqdm.github.io/] |

**Installation:**
```bash
python3 -m pip install --upgrade huggingface_hub requests tqdm
cargo add ratatui crossterm tui-input inquire
```

**Version verification:** before relying on any recommended package version, verify the registry value against `cargo search` or `python3 -m pip index versions`; training data and local pins can lag current releases. [VERIFIED: `cargo search` and `python3 -m pip index versions` in this session]

## Architecture Patterns

### System Architecture Diagram

```text
User
  |
  v
Launcher / startup prompt
  |
  +--> discover local models in models/ [VERIFIED: scripts/config.py]
  |       |
  |       +--> choose model / refresh catalog
  |       |
  |       +--> write runtime handoff env state [VERIFIED: scripts/start_server.py; start.py]
  |
  +--> optional GGUF download flow
  |       |
  |       +--> list HF repo tree / available .gguf files [CITED: huggingface hub docs]
  |       |
  |       +--> download selected file with progress [CITED: huggingface hub docs; CITED: tqdm docs]
  |       |
  |       +--> stage + checksum verify + activate into models/ [VERIFIED: scripts/model_install.py]
  |
  +--> TUI session
          |
          +--> input, ghost suggestion, slash-command registry [VERIFIED: agent-rs/src/tui.rs; agent-rs/src/tui/commands.rs]
          |
          +--> /gsd plan or /gsd execute actions [VERIFIED: agent-rs/src/main.rs]
          |
          +--> model/runtime status snapshots [VERIFIED: agent-rs/src/main.rs]
```

### Recommended Project Structure
```text
scripts/
├── config.py        # model discovery and runtime profile assembly [VERIFIED]
├── model_install.py # trusted install, staging, integrity checks [VERIFIED]
├── download_model.py# interactive HF download flow [VERIFIED]
└── start_server.py  # runtime model resolution and server launch [VERIFIED]

agent-rs/src/
├── tui.rs           # TUI input, ghost suggestions, slash commands [VERIFIED]
├── tui/             # state, commands, approval, themes, events [VERIFIED]
└── main.rs          # orchestrator loop and context snapshot emission [VERIFIED]

tests/
├── test_model_install.py # install and integrity behavior [VERIFIED]
└── test_start_server_runtime_profile.py # runtime env override behavior [VERIFIED]
```

### Pattern 1: Single source of truth for model choice
**What:** resolve the active model once, then pass it forward through the launcher/server boundary instead of recomputing a second catalog later. [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/start_server.py`, `start.py`]
**When to use:** startup selection, server launch, and any post-install refresh path. [VERIFIED: repo code inspection of `scripts/model_install.py`, `scripts/config.py`]
**Example:**
```python
# Source: repo code pattern in scripts/config.py and scripts/start_server.py
model_entry = build_model_entry(model_name, detected_vram_gb)
os.environ["HELIX_MODEL_NAME"] = model_entry["model_name"]
os.environ["HELIX_MODEL_PATH"] = model_entry["path"]
```

### Pattern 2: Stateful terminal picker
**What:** keep the selected row/item in widget state and re-render it, rather than rebuilding the menu from scratch on every keypress. [CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html; CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html]
**When to use:** if Phase 05 keeps model selection inside the full-screen Rust TUI instead of using a startup prompt. [VERIFIED/CITED sources above]
**Example:**
```rust
// Source: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html
frame.render_stateful_widget(list, area, &mut list_state);
```

### Pattern 3: Terminal prompt with skippable selection
**What:** use a prompt that can be skipped or confirmed without leaving the terminal workflow. [CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html]
**When to use:** startup model choice or lightweight confirmation flows. [VERIFIED: repo already uses inquire in `agent-rs/src/tui/approval.rs`]
**Example:**
```rust
// Source: https://docs.rs/inquire/latest/inquire/struct.Select.html
let selected = Select::new("Choose a model", models).prompt_skippable()?;
```

### Anti-Patterns to Avoid
- **Parallel model registries:** do not keep a separate Python catalog, Rust catalog, and launcher-specific catalog; they will drift. [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/start_server.py`, `start.py`]
- **Import-time discovery only:** do not assume `MODEL_CATALOG` stays correct after install unless the discovery step is refreshed. [VERIFIED: repo code inspection of `scripts/config.py`]
- **Raw HTTP file-tree scraping for every download:** use the Hugging Face client helpers for repo metadata and file retrieval instead of manual URL logic. [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|--------------|-----|
| Hugging Face repo browsing and file download | Custom API pagination, file URL construction, and retry logic | `huggingface_hub` repo metadata and download helpers | The Hub client already handles metadata, auth-aware access, cache behavior, and download semantics. [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download] |
| Progress reporting for large GGUF downloads | A bespoke terminal progress meter | `tqdm` | `tqdm` already provides tested progress UX for long-running transfers. [CITED: https://tqdm.github.io/] |
| Startup model selection menu | A custom one-off text parser | `inquire::Select` or ratatui stateful widgets | The repo already depends on prompt and TUI libraries that solve selection state and keyboard navigation. [VERIFIED: `agent-rs/Cargo.toml`; CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html; CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html] |
| Model integrity verification | A new checksum scheme | Existing SHA-256 validation in the install path | The install flow already verifies integrity before activation. [VERIFIED: repo code inspection of `scripts/model_install.py`] |
| GSD command suggestions | A second parser or hint engine | Existing slash-command registry and ghost suggestion logic | The TUI already has slash commands and autofill behavior to extend. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`, `agent-rs/src/tui/commands.rs`] |

**Key insight:** this phase should narrow, not widen, the number of places that understand "which model is active". [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/start_server.py`, `start.py`]

## Common Pitfalls

### Pitfall 1: Stale catalog after install
**What goes wrong:** a model is downloaded into `models/`, but the process still uses the old import-time catalog. [VERIFIED: repo code inspection of `scripts/config.py`]
**Why it happens:** `MODEL_CATALOG` is created when `scripts/config.py` is imported, so callers that do not refresh discovery continue to see stale state. [VERIFIED: repo code inspection of `scripts/config.py`]
**How to avoid:** reload or re-scan model discovery after install before rendering the next picker or launching the server. [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/model_install.py`]
**Warning signs:** the newly downloaded file is present on disk but absent from the next startup menu. [VERIFIED: repo code inspection of `scripts/config.py`]

### Pitfall 2: Diverging runtime handoff
**What goes wrong:** the UI shows one model while the server starts with another. [VERIFIED: repo code inspection of `start.py`, `scripts/start_server.py`]
**Why it happens:** selection state is not carried forward through the same env/runtime boundary that the launcher uses. [VERIFIED: repo code inspection of `start.py`, `scripts/start_server.py`]
**How to avoid:** always write the chosen model into the same runtime handoff variables before spawning the server. [VERIFIED: repo code inspection of `start.py`, `scripts/start_server.py`]
**Warning signs:** runtime status snapshots and startup prompts disagree about the active model. [VERIFIED: repo code inspection of `agent-rs/src/main.rs`]

### Pitfall 3: Replacing a simple prompt with a custom widget stack too early
**What goes wrong:** the phase spends time rebuilding terminal UI machinery when a prompt would have been enough. [CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html]
**Why it happens:** the full-screen TUI already exists, so it is tempting to force every new selection into that surface. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`]
**How to avoid:** choose the smallest selection surface that meets the UX requirement, then keep the selection state in one place. [CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html; CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html]
**Warning signs:** duplicated model lists, duplicated key handling, or a startup picker that has to be reimplemented in more than one language. [VERIFIED: repo code inspection of `scripts/config.py`, `agent-rs/src/tui.rs`, `start.py`]

### Pitfall 4: Overwriting GSD autofill intent
**What goes wrong:** ghost suggestions stop matching the last explicit `/gsd` command. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`]
**Why it happens:** the suggestion logic depends on `last_gsd_command` and the current command registry. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`, `agent-rs/src/tui/commands.rs`]
**How to avoid:** keep suggestion behavior data-driven from the command registry and preserve the fallback path to typed input. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`, `agent-rs/src/tui/commands.rs`]
**Warning signs:** Tab or Right no longer accepts the expected `/gsd plan` and `/gsd execute` suggestions. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`]

## Code Examples

Verified patterns from official sources:

### Startup selection with a skippable prompt
```rust
// Source: https://docs.rs/inquire/latest/inquire/struct.Select.html
use inquire::Select;

let selected_model = Select::new("Choose a model", model_names)
    .prompt_skippable()?;
```

### Stateful in-TUI selection
```rust
// Source: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html
use ratatui::widgets::ListState;

frame.render_stateful_widget(model_list, area, &mut list_state);
```

### Hugging Face GGUF download flow
```python
# Source: https://huggingface.co/docs/huggingface_hub/package_reference/file_download
from huggingface_hub import hf_hub_download

path = hf_hub_download(
    repo_id="org/model",
    filename="model.gguf",
    revision="main",
    local_dir="models",
)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual HTTP tree scraping for every model download | Hugging Face Hub metadata and download helpers | Current HF Hub docs and repo code path [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download] | Less custom error handling, better auth/cache behavior, and smaller maintenance surface. |
| Static model state used as the only registry | Filesystem-backed discovery plus refreshable install flow | Current repo code [VERIFIED: `scripts/config.py`, `scripts/model_install.py`] | Newly installed models can appear without manual code edits. |
| Stateless terminal menus | Stateful widgets or skippable terminal prompts | Current docs for `ratatui` and `inquire` [CITED: https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html; CITED: https://docs.rs/inquire/latest/inquire/struct.Select.html] | Keyboard navigation and selection state become easier to keep consistent. |

**Deprecated/outdated:**
- Hand-written progress meters for model download are lower value than `tqdm` for this phase. [CITED: https://tqdm.github.io/]
- A second model catalog in Rust is outdated if Python already owns discovery and install activation. [VERIFIED: repo code inspection of `scripts/config.py`, `scripts/model_install.py`, `scripts/start_server.py`]

## Assumptions Log

> If this table is empty: all claims in this research were verified or cited, and no user confirmation is needed.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| None | None | None | None |

## Open Questions

1. **Where should startup model selection live?**
   - What we know: `start.py` already owns a model chooser, and the Rust TUI already owns interactive prompts and selection state. [VERIFIED: repo code inspection of `start.py`, `agent-rs/src/tui.rs`]
   - What's unclear: whether Phase 05 wants the startup picker in Python, in Rust, or in both places with one canonical owner. [UNKNOWN]
   - Recommendation: pick one owner and have the other layer only consume the chosen model, not re-decide it. [RECOMMENDATION]

2. **Should Hugging Face downloads support gated/private repos in this phase?**
   - What we know: the Hub client docs cover repo metadata and download helpers that can participate in auth-aware flows. [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download]
   - What's unclear: whether the user-facing UX must expose token handling or only support public repos. [UNKNOWN]
   - Recommendation: keep the downloader auth-capable at the library boundary, but do not add a custom credential store unless the phase requires it. [RECOMMENDATION]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Python 3 | `scripts/download_model.py`, Python tests, launcher scripts | ✓ | 3.14.4 | — |
| Cargo / Rust toolchain | `agent-rs` build and tests | ✓ | 1.95.0 | — |
| Git | repo operations / commits | ✓ | 2.54.0 | — |
| pytest | Python test execution | ✓ | 9.0.3 | — |
| huggingface_hub | HF model metadata and download support | ✓ | 1.8.0 installed; 1.12.0 latest | — |
| requests | Python HTTP client for current downloader path | ✓ | 2.32.5 installed; 2.33.1 latest | — |
| tqdm | download progress UI | ✓ | 4.67.3 | — |

**Missing dependencies with no fallback:**
- None. [VERIFIED: environment audit in this session]

**Missing dependencies with fallback:**
- None. [VERIFIED: environment audit in this session]

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | pytest 9.0.3 and Rust unit tests via `cargo test` [VERIFIED: `python3 -m pytest --version`; VERIFIED: `agent-rs/Cargo.toml`] |
| Config file | none in the repo root for the helix-agent test suite; nested `llama.cpp/tools/server/tests/pytest.ini` is unrelated to this phase [VERIFIED: `file_search`]
| Quick run command | `python3 -m pytest tests/test_model_install.py -x` |
| Full suite command | `python3 -m pytest tests && cargo test -p agent-rs` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| P05-01 | Discover local GGUF models from `models/` and refresh the catalog after install | unit | `python3 -m pytest tests/test_model_install.py -x` | ✅ `tests/test_model_install.py` exists; discovery-refresh coverage still needs to be added. [VERIFIED: repo search and file read] |
| P05-02 | Preserve runtime model handoff into the server launch path | integration | `python3 -m pytest tests/test_start_server_runtime_profile.py -x` | ✅ `tests/test_start_server_runtime_profile.py` exists. [VERIFIED: repo search] |
| P05-03 | Keep TUI ghost suggestion aligned with `/gsd` command state | unit | `cargo test -p agent-rs up_scroll_refresh_gsd_autofill -- --nocapture` | ✅ covered by an existing Rust unit test in `agent-rs/src/tui.rs`. [VERIFIED: file read] |
| P05-04 | Render a startup model picker without breaking keyboard selection state | unit / integration | `cargo test -p agent-rs tui::state -- --nocapture` | ✅ Rust TUI test coverage exists; a picker-specific test may still need to be added. [VERIFIED: file search and file read] |
| P05-05 | Download GGUF files from Hugging Face with staging and checksum verification | unit / integration | `python3 -m pytest tests/test_model_install.py -x` | ✅ install/integrity tests exist; downloader-specific coverage is still a gap. [VERIFIED: repo search and file read] |

### Sampling Rate
- **Per task commit:** run the narrowest slice for the file you changed, usually `python3 -m pytest tests/test_model_install.py -x` for Python and `cargo test -p agent-rs <test-name> -- --nocapture` for Rust. [VERIFIED: test files and inline Rust unit tests]
- **Per wave merge:** run `python3 -m pytest tests && cargo test -p agent-rs`. [VERIFIED: repo test layout]
- **Phase gate:** keep both suites green before handing the phase to `/gsd-verify-work`. [VERIFIED: workflow.nyquist_validation is enabled in `.planning/config.json`]

### Wave 0 Gaps
- [ ] `tests/test_download_model.py` - downloader-specific coverage for the GGUF selection flow. [VERIFIED: repo search found no matching file]
- [ ] `tests/test_startup_model_picker.py` or a Rust equivalent - picker behavior is not yet isolated as a dedicated test target. [VERIFIED: repo search and file read]
- [ ] `tests/test_model_install.py` - add a discovery-refresh case that proves a new file appears after reloading `scripts/config.py`. [VERIFIED: repo code inspection of `scripts/config.py`]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | yes if gated/private Hugging Face repos are in scope; otherwise no | pass auth through the Hugging Face client boundary instead of inventing a custom credential path. [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api; CITED: https://huggingface.co/docs/huggingface_hub/package_reference/file_download] |
| V3 Session Management | no | not a sessioned web flow. [VERIFIED: repo code inspection] |
| V4 Access Control | yes for trusted-model allowlists and repo gating | keep the trusted spec validation and avoid bypassing the trusted install path. [VERIFIED: repo code inspection of `scripts/model_install.py`] |
| V5 Input Validation | yes | validate model names, repo IDs, revision pins, and checksum values before download or activation. [VERIFIED: repo code inspection of `scripts/model_install.py`, `scripts/download_model.py`] |
| V6 Cryptography | yes | keep SHA-256 verification in the install pipeline; do not hand-roll a weaker integrity check. [VERIFIED: repo code inspection of `scripts/model_install.py`] |

### Known Threat Patterns for Rust TUI + Python downloader stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious model path or filename injection | Tampering | sanitize and validate paths before activating a model file. [VERIFIED: repo code inspection of `scripts/model_install.py`] |
| Downloading an unexpected artifact from a repo | Tampering | require pinned filenames/revisions and verify SHA-256 before activation. [VERIFIED: repo code inspection of `scripts/model_install.py`] |
| Prompt poisoning via suggestion history | Spoofing | keep ghost suggestions constrained to the slash-command registry and last explicit GSD command. [VERIFIED: repo code inspection of `agent-rs/src/tui.rs`, `agent-rs/src/tui/commands.rs`] |
| Private repo token leakage in logs | Information Disclosure | keep auth handling in the library boundary and avoid printing secrets in downloader errors. [CITED: https://huggingface.co/docs/huggingface_hub/package_reference/hf_api] |

## Sources

### Primary (HIGH confidence)
- `cargo search inquire --limit 1`, `cargo search ratatui --limit 1`, `cargo search crossterm --limit 1`, `cargo search tui-input --limit 1` - current crate versions and availability. [VERIFIED]
- `python3 -m pip index versions huggingface_hub`, `python3 -m pip index versions requests`, `python3 -m pip index versions tqdm` - current PyPI versions and installed versions. [VERIFIED]
- `agent-rs/Cargo.toml`, `scripts/config.py`, `scripts/model_install.py`, `scripts/download_model.py`, `scripts/start_server.py`, `start.py`, `agent-rs/src/tui.rs`, `agent-rs/src/tui/state.rs`, `agent-rs/src/tui/approval.rs`, `agent-rs/src/main.rs` - repo architecture and current behavior. [VERIFIED]
- `tests/test_model_install.py`, `tests/test_start_server_runtime_profile.py` - existing Python validation coverage. [VERIFIED]

### Secondary (MEDIUM confidence)
- https://docs.rs/inquire/latest/inquire/struct.Select.html - terminal prompt selection API. [CITED]
- https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html - stateful list rendering. [CITED]
- https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html - stateful table rendering. [CITED]
- https://huggingface.co/docs/huggingface_hub/package_reference/file_download - file download helpers. [CITED]
- https://huggingface.co/docs/huggingface_hub/package_reference/hf_api - repo metadata and listing helpers. [CITED]
- https://tqdm.github.io/ - progress bar library reference. [CITED]

### Tertiary (LOW confidence)
- None. [VERIFIED: no unverified web-search-only claims were needed]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - current registry versions and repo pins were checked in this session. [VERIFIED]
- Architecture: HIGH - the repo code shows the split between Python model lifecycle code and Rust TUI UX code. [VERIFIED]
- Pitfalls: HIGH - each pitfall maps to a concrete repo behavior or a documented library pattern. [VERIFIED/CITED]

**Research date:** 2026-04-26
**Valid until:** 2026-05-26
