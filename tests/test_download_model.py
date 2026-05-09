"""
Tests for MOD-03: Hugging Face Downloader

Verifies list_repo_files() and download_file() from scripts/download_model.py
"""
import sys
from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest

PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))


class TestListRepoFiles:
    """Test repository file listing functionality."""

    def test_list_repo_files_exists(self):
        """Function exists and is importable."""
        from download_model import list_repo_files
        assert callable(list_repo_files)

    @patch("download_model._api")
    @patch("download_model._repo_info")
    def test_returns_gguf_files_only(self, mock_repo_info, mock_api):
        """Returns only .gguf files from the repository."""
        from download_model import list_repo_files

        # Mock repo info with both gguf and non-gguf files
        mock_sibling_gguf = MagicMock()
        mock_sibling_gguf.rfilename = "model-Q4_K_M.gguf"
        mock_sibling_gguf.size = 5000000000
        mock_sibling_gguf.lfs = MagicMock(sha256="abc123")

        mock_sibling_txt = MagicMock()
        mock_sibling_txt.rfilename = "README.txt"
        mock_sibling_txt.size = 1000
        mock_sibling_txt.lfs = None

        mock_repo_obj = MagicMock()
        mock_repo_obj.siblings = [mock_sibling_gguf, mock_sibling_txt]
        mock_repo_obj.sha = "def456"
        mock_repo_info.return_value = mock_repo_obj

        result = list_repo_files("test/repo")

        # Should only return the GGUF file
        assert len(result) == 1
        assert result[0]["filename"] == "model-Q4_K_M.gguf"

    @patch("download_model._api")
    @patch("download_model._repo_info")
    def test_includes_file_metadata(self, mock_repo_info, mock_api):
        """Returns file metadata including size and sha256."""
        from download_model import list_repo_files

        mock_sibling = MagicMock()
        mock_sibling.rfilename = "test-model.gguf"
        mock_sibling.size = 1234567890
        mock_sibling.lfs = MagicMock(sha256="deadbeef12345678")

        mock_repo_obj = MagicMock()
        mock_repo_obj.siblings = [mock_sibling]
        mock_repo_obj.sha = "revision123"
        mock_repo_info.return_value = mock_repo_obj

        result = list_repo_files("test/repo")

        assert len(result) == 1
        assert result[0]["size"] == 1234567890
        assert result[0]["sha256"] == "deadbeef12345678"
        assert result[0]["repo_id"] == "test/repo"
        assert result[0]["repo_revision"] == "revision123"

    @patch("download_model._api")
    @patch("download_model._repo_info")
    def test_handles_missing_repo(self, mock_repo_info, mock_api):
        """Handles missing/non-existent repositories gracefully."""
        from download_model import list_repo_files

        mock_repo_info.side_effect = Exception("Repository not found")

        with pytest.raises(Exception):
            list_repo_files("nonexistent/repo")

    @patch("download_model._api")
    @patch("download_model._repo_info")
    def test_handles_no_gguf_files(self, mock_repo_info, mock_api):
        """Raises ValueError when no GGUF files found."""
        from download_model import list_repo_files

        mock_sibling = MagicMock()
        mock_sibling.rfilename = "README.md"
        mock_sibling.size = 1000

        mock_repo_obj = MagicMock()
        mock_repo_obj.siblings = [mock_sibling]
        mock_repo_info.return_value = mock_repo_obj

        with pytest.raises(ValueError, match="No .gguf files found"):
            list_repo_files("test/repo")


class TestDownloadFile:
    """Test download functionality."""

    def test_download_file_exists(self):
        """Function exists and is importable."""
        from download_model import download_file
        assert callable(download_file)


class TestRepoNormalization:
    """Test repository ID normalization."""

    def test_normalize_with_huggingface_co_url(self):
        """Strips huggingface.co prefix."""
        from download_model import normalize_repo_id

        result = normalize_repo_id("https://huggingface.co/TheBloke/Mistral-7B-v0.1-GGUF")
        assert result == "TheBloke/Mistral-7B-v0.1-GGUF"

    def test_normalize_with_hf_co_url(self):
        """Strips hf.co prefix."""
        from download_model import normalize_repo_id

        result = normalize_repo_id("hf.co/TheBloke/Mistral-7B-v0.1-GGUF")
        assert result == "TheBloke/Mistral-7B-v0.1-GGUF"

    def test_normalize_plain_repo_id(self):
        """Leaves plain repo ID unchanged."""
        from download_model import normalize_repo_id

        result = normalize_repo_id("TheBloke/Mistral-7B-v0.1-GGUF")
        assert result == "TheBloke/Mistral-7B-v0.1-GGUF"

    def test_normalize_strips_trailing_slash(self):
        """Removes trailing slashes."""
        from download_model import normalize_repo_id

        result = normalize_repo_id("TheBloke/Mistral-7B-v0.1-GGUF/")
        assert result == "TheBloke/Mistral-7B-v0.1-GGUF"


class TestFormatSize:
    """Test human-readable size formatting."""

    def test_format_bytes(self):
        """Formats bytes correctly."""
        from download_model import format_size

        assert format_size(500) == "500.00 B"

    def test_format_kilobytes(self):
        """Formats KB correctly."""
        from download_model import format_size

        assert format_size(1024) == "1.00 KB"

    def test_format_megabytes(self):
        """Formats MB correctly."""
        from download_model import format_size

        assert format_size(1024 * 1024) == "1.00 MB"

    def test_format_gigabytes(self):
        """Formats GB correctly."""
        from download_model import format_size

        assert format_size(1024 * 1024 * 1024) == "1.00 GB"


class TestListReposByTag:
    """Test repository search by tag."""

    def test_list_repos_by_tag_exists(self):
        """Function exists and is importable."""
        from download_model import list_repos_by_tag
        assert callable(list_repos_by_tag)