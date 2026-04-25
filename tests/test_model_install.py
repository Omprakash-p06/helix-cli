import pytest
import shutil
import hashlib
from pathlib import Path
import os
import sys

# Ensure the scripts directory is in sys.path
PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from model_install import (
    verify_model_integrity,
    resolve_model_ref,
    install_model_spec,
    MODELS_DIR,
    STAGING_DIR
)

@pytest.fixture
def temp_models_dir(tmp_path):
    """Provides a temporary models directory for testing."""
    models_dir = tmp_path / "models"
    staging_dir = models_dir / ".staging"
    models_dir.mkdir()
    staging_dir.mkdir()
    
    # Monkeypatch original paths in model_install if needed
    return models_dir, staging_dir

def test_verify_model_integrity(tmp_path):
    # Create a dummy model file
    model_file = tmp_path / "test_model.gguf"
    content = b"fake model content"
    model_file.write_bytes(content)
    
    expected_hash = hashlib.sha256(content).hexdigest()
    assert verify_model_integrity(model_file, expected_hash) is True
    assert verify_model_integrity(model_file, "wrong_hash") is False

def test_resolve_model_ref():
    # Updated to use models actually in TRUSTED_MODELS
    assert resolve_model_ref("qwen-3.6-27b-moe") is not None
    assert resolve_model_ref("non_existent_model") is None
    assert resolve_model_ref("Qwen/Qwen3.6-27B-Instruct-GGUF") is not None

def test_install_model_spec_checksum_failure(tmp_path, monkeypatch):
    # Use tmp_path for MODELS_DIR and STAGING_DIR
    monkeypatch.setattr("model_install.MODELS_DIR", tmp_path / "models")
    monkeypatch.setattr("model_install.STAGING_DIR", tmp_path / "models" / ".staging")
    
    (tmp_path / "models").mkdir()
    (tmp_path / "models" / ".staging").mkdir()
    
    # Mock download to return a fake file
    dummy_model = tmp_path / "models" / ".staging" / "fail_model.gguf"
    dummy_model.write_bytes(b"tampered content")
    
    monkeypatch.setattr("model_install.download_model_to_staging", lambda spec, **kwargs: dummy_model)
    
    fake_sha = "a" * 64
    spec = {
        "name": "Tampered Model",
        "repo": "some/repo",
        "filename": "fail_model.gguf",
        "revision": "deadbeef" * 5, # 40 char pinned revision
        "sha256": fake_sha
    }
    
    # This should fail the integrity check and return False
    assert install_model_spec(spec) is False
    assert not (tmp_path / "models" / "fail_model.gguf").exists()

def test_install_model_spec_success(tmp_path, monkeypatch):
    monkeypatch.setattr("model_install.MODELS_DIR", tmp_path / "models")
    monkeypatch.setattr("model_install.STAGING_DIR", tmp_path / "models" / ".staging")
    
    (tmp_path / "models").mkdir()
    (tmp_path / "models" / ".staging").mkdir()
    
    content = b"good content"
    good_hash = hashlib.sha256(content).hexdigest()
    
    dummy_model = tmp_path / "models" / ".staging" / "good_model.gguf"
    dummy_model.write_bytes(content)
    
    monkeypatch.setattr("model_install.download_model_to_staging", lambda spec, **kwargs: dummy_model)
    
    spec = {
        "name": "Good Model",
        "repo": "some/repo",
        "filename": "good_model.gguf",
        "revision": "12345678" * 5, # 40 char pinned revision
        "sha256": good_hash
    }
    
    # This should pass integrity check and move the file
    assert install_model_spec(spec) is True
    assert (tmp_path / "models" / "good_model.gguf").exists()
    assert (tmp_path / "models" / "good_model.gguf").read_bytes() == content
