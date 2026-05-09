"""
Tests for MOD-02: Dynamic Model Discovery

Verifies scan_models_directory() behavior from scripts/config.py
"""
import sys
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from config import scan_models_directory, ModelEntry


class TestScanModelsDirectory:
    """Test scan_models_directory() returns proper data structures."""

    def test_returns_list_of_model_entries(self, tmp_path):
        """scan_models_directory returns a list."""
        # Create a temp directory with no models
        (tmp_path / "models").mkdir()
        result = scan_models_directory(str(tmp_path / "models"))
        assert isinstance(result, list)

    def test_finds_gguf_files(self, tmp_path):
        """Discovers .gguf files in the models directory."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()

        # Create fake GGUF files
        (models_dir / "test-model-7b.Q4_K_M.gguf").write_text("fake model")
        (models_dir / "test-model-13b.Q8_0.gguf").write_text("fake model")

        result = scan_models_directory(str(models_dir))
        assert len(result) == 2
        names = [entry.name for entry in result]
        assert "test-model-7b.Q4_K_M" in names
        assert "test-model-13b.Q8_0" in names

    def test_returns_model_entry_with_required_fields(self, tmp_path):
        """Each entry has name, path, size_mb, vram_estimate_gb."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()
        (models_dir / "small-model.gguf").write_bytes(b"x" * 1024 * 1024)  # 1MB

        result = scan_models_directory(str(models_dir))
        assert len(result) == 1
        entry = result[0]

        assert isinstance(entry, ModelEntry)
        assert entry.name == "small-model"
        assert entry.path.endswith("small-model.gguf")
        assert entry.size_mb > 0
        assert entry.vram_estimate_gb > 0

    def test_handles_empty_directory(self, tmp_path):
        """Handles empty models/ directory gracefully."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()
        # No GGUF files

        result = scan_models_directory(str(models_dir))
        assert result == []
        # No crash, returns empty list

    def test_handles_nonexistent_directory(self, tmp_path):
        """Returns empty list when directory doesn't exist."""
        result = scan_models_directory(str(tmp_path / "nonexistent"))
        assert result == []

    def test_sorts_by_parameter_count(self, tmp_path):
        """Results sorted by parameter count (ascending)."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()

        # Files with parameter hints in name
        (models_dir / "model-7b.gguf").write_text("7b")
        (models_dir / "model-70b.gguf").write_text("70b")
        (models_dir / "model-3b.gguf").write_text("3b")

        result = scan_models_directory(str(models_dir))
        names = [entry.name for entry in result]

        # 3b should come first (lowest params), then 7b, then 70b
        assert names.index("model-3b") < names.index("model-7b")
        assert names.index("model-7b") < names.index("model-70b")

    def test_handles_no_parameter_hint_in_filename(self, tmp_path):
        """Files without parameter count sorted by size."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()

        # One with param hint, one without
        (models_dir / "model-7b.gguf").write_bytes(b"a" * 100)
        (models_dir / "unnamed.gguf").write_bytes(b"b" * 200)

        result = scan_models_directory(str(models_dir))
        # Both should appear, no crash
        assert len(result) == 2


class TestStartupModelPicker:
    """Test integration with startup model picker logic."""

    def test_single_model_available(self, tmp_path):
        """When only 1 model exists, selection should be implicit."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()
        (models_dir / "only-model.gguf").write_text("content")

        result = scan_models_directory(str(models_dir))
        assert len(result) == 1

    def test_multiple_models_available(self, tmp_path):
        """When 2+ models exist, user should be offered choice."""
        models_dir = tmp_path / "models"
        models_dir.mkdir()

        (models_dir / "model-a-7b.gguf").write_text("a")
        (models_dir / "model-b-13b.gguf").write_text("b")

        result = scan_models_directory(str(models_dir))
        # Both models should be discoverable for selection
        assert len(result) >= 2
        names = [entry.name for entry in result]
        assert any("model-a-7b" in n for n in names)
        assert any("model-b-13b" in n for n in names)