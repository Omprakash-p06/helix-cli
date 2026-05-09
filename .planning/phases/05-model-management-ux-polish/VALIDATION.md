# Phase 05 Validation Map

## Phase: 05-model-management-ux-polish
**Audited:** 2025-05-08
**Test Framework:** pytest 9.0.3

---

## Gap Resolution Summary

| Gap ID | Requirement | Test File | Command | Status |
|--------|-------------|-----------|---------|--------|
| 1 | MOD-02 (Dynamic Model Discovery) | test_model_discovery.py | `python3 -m pytest tests/test_model_discovery.py -v` | green |
| 2 | MOD-03 (HF Downloader) | test_download_model.py | `python3 -m pytest tests/test_download_model.py -v` | green |

---

## Test Coverage Details

### Gap 1: MOD-02 (Dynamic Model Discovery)

**Function Tested:** `scan_models_directory()` in `scripts/config.py`

| Test | Coverage |
|------|----------|
| test_returns_list_of_model_entries | Returns list type |
| test_finds_gguf_files | Discovers .gguf files |
| test_returns_model_entry_with_required_fields | Validates ModelEntry fields |
| test_handles_empty_directory | Graceful empty handling |
| test_handles_nonexistent_directory | Missing dir handling |
| test_sorts_by_parameter_count | Parameter-based sorting |
| test_handles_no_parameter_hint_in_filename | Files without param hints |
| test_single_model_available | Single model scenario |
| test_multiple_models_available | 2+ model selection |

**Total Tests:** 9

### Gap 2: MOD-03 (HF Downloader)

**Functions Tested:** `list_repo_files()`, `download_file()`, `normalize_repo_id()`, `format_size()` in `scripts/download_model.py`

| Test | Coverage |
|------|----------|
| test_list_repo_files_exists | Function importability |
| test_returns_gguf_files_only | GGUF-only filtering |
| test_includes_file_metadata | Size/sha256 metadata |
| test_handles_missing_repo | Error handling |
| test_handles_no_gguf_files | No-GGUF error |
| test_download_file_exists | Function importability |
| test_normalize_with_huggingface_co_url | URL stripping |
| test_normalize_with_hf_co_url | hf.co prefix stripping |
| test_normalize_plain_repo_id | Plain ID handling |
| test_normalize_strips_trailing_slash | Trailing slash removal |
| test_format_bytes | Byte formatting |
| test_format_kilobytes | KB formatting |
| test_format_megabytes | MB formatting |
| test_format_gigabytes | GB formatting |
| test_list_repos_by_tag_exists | Tag search function |

**Total Tests:** 15

---

## Verification Commands

```bash
# Run model discovery tests
python3 -m pytest tests/test_model_discovery.py -v

# Run HF downloader tests
python3 -m pytest tests/test_download_model.py -v

# Run all phase tests
python3 -m pytest tests/test_qwen_config.py tests/test_model_install.py tests/test_model_discovery.py tests/test_download_model.py -v

# Verify scan_models_directory works
python3 -c "from scripts.config import scan_models_directory; models = scan_models_directory(); print(f'Found {len(models)} models')"

# Verify list_repo_files import works
python3 -c "from scripts.download_model import list_repo_files; print('OK')"
```

---

## Files Created

- `tests/test_model_discovery.py` - 9 tests for MOD-02
- `tests/test_download_model.py` - 15 tests for MOD-03