import os
import sys
from pathlib import Path

PROJECT_DIR = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_DIR / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))


def test_invalid_numeric_overrides_are_ignored(monkeypatch):
    import config
    import start_server

    config.BATCH_SIZE = 512
    config.UBATCH_SIZE = 256
    config.CPU_THREADS = 8
    config.CONTEXT_SIZE = 8192

    monkeypatch.setenv("HELIX_BATCH_SIZE", "not-a-number")
    monkeypatch.setenv("HELIX_UBATCH_SIZE", "NaN")
    monkeypatch.setenv("HELIX_CPU_THREADS", "oops")
    monkeypatch.setenv("HELIX_CONTEXT_SIZE", "invalid")

    start_server.apply_runtime_overrides()

    assert config.BATCH_SIZE == 512
    assert config.UBATCH_SIZE == 256
    assert config.CPU_THREADS == 8
    assert config.CONTEXT_SIZE == 8192


def test_fallback_runtime_overrides_apply(monkeypatch):
    import config
    import start_server

    config.FALLBACK_BACKEND_HINT = "cpu"
    config.FALLBACK_GPU_LAYERS = 0

    monkeypatch.setenv("HELIX_FALLBACK_BACKEND_HINT", "vulkan")
    monkeypatch.setenv("HELIX_FALLBACK_GPU_LAYERS", "12")

    start_server.apply_runtime_overrides()

    assert config.FALLBACK_BACKEND_HINT == "vulkan"
    assert config.FALLBACK_GPU_LAYERS == 12


def test_runtime_profile_flag_is_non_destructive(monkeypatch):
    import config
    import start_server

    config.BACKEND_HINT = "cpu"
    monkeypatch.setenv("HELIX_RUNTIME_PROFILE", "balanced_cpu")

    start_server.apply_runtime_overrides()

    # Profile selection itself should not mutate backend without explicit overrides.
    assert config.BACKEND_HINT == "cpu"
