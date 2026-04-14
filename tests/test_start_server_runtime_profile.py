import os
import subprocess
import sys
import pytest
from pathlib import Path

# Ensure we can import from scripts
PROJECT_DIR = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_DIR / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

def test_start_server_env_overrides():
    """Verify that HELIX_* env vars override config.py defaults in the launcher command."""
    os.environ["HELIX_RUNTIME_PROFILE"] = "latency_cpu"
    os.environ["HELIX_BATCH_SIZE"] = "128"
    os.environ["HELIX_UBATCH_SIZE"] = "64"
    os.environ["HELIX_CPU_THREADS"] = "4"
    os.environ["HELIX_CONTEXT_SIZE"] = "2048"
    os.environ["HELIX_BACKEND_HINT"] = "cpu"
    os.environ["HELIX_GPU_LAYERS"] = "0"
    
    # We use a mock check to avoid actually starting the server
    # By running with --help or a non-existent model we can see the command it WOULD run
    # if we capture stdout (we added print(f"  Command: {' '.join(cmd)}") in start_server.py)
    
    # Actually, we can just import apply_runtime_overrides and check the config object
    import config
    import start_server
    
    # Reset config to known state (simulating fresh import)
    config.BATCH_SIZE = 512
    config.CONTEXT_SIZE = 8192
    
    # Apply overrides
    start_server.apply_runtime_overrides()
    
    # These should now match the env vars
    assert config.BATCH_SIZE == 128
    assert config.UBATCH_SIZE == 64
    assert config.CPU_THREADS == 4
    assert config.CONTEXT_SIZE == 2048
    assert config.BACKEND_HINT == "cpu"
    assert config.GPU_LAYERS == 0

def test_start_server_partial_overrides():
    """Verify that partial overrides only affect specified variables."""
    import config
    import start_server
    
    # Reset config
    config.BATCH_SIZE = 512
    config.CONTEXT_SIZE = 8192
    
    # Mock env
    os.environ["HELIX_CONTEXT_SIZE"] = "4096"
    if "HELIX_BATCH_SIZE" in os.environ: del os.environ["HELIX_BATCH_SIZE"]
    
    start_server.apply_runtime_overrides()
    
    assert config.CONTEXT_SIZE == 4096
    assert config.BATCH_SIZE == 512 # Remained default
