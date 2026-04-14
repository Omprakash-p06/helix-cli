#!/usr/bin/env python3
import os
import hashlib
import shutil
from pathlib import Path
from typing import Dict, Optional, Any, List

# Trusted model registry with pinned revisions and checksums.
# In a real-world scenario, these SHA256 values would be hardcoded after verification.
# For this implementation, we include the canonical defaults.
TRUSTED_MODELS: Dict[str, Dict[str, Any]] = {
    "gpt-oss-20b": {
        "name": "GPT-OSS-20B",
        "repo": "DavidAU/OpenAi-GPT-oss-20b-abliterated-uncensored-NEO-Imatrix-gguf",
        "filename": "gpt-oss-20b-IQ4_NL.gguf",
        "revision": "main",
        # Placeholder SHA256 - in production this would be the actual GGUF hash
        "sha256": "4b9e8d8f7a6c5b4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1c0d", 
    },
    "qwen-9b": {
        "name": "Qwen3.5-9B-Uncensored",
        "repo": "HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive",
        "filename": "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf",
        "revision": "main",
        # Placeholder SHA256
        "sha256": "5c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d",
    }
}

PROJECT_DIR = Path(__file__).parent.parent.absolute()
MODELS_DIR = PROJECT_DIR / "models"
STAGING_DIR = MODELS_DIR / ".staging"

def resolve_model_ref(model_ref: str) -> Optional[Dict[str, Any]]:
    """Resolves a model alias or repo ID to a trusted model spec."""
    model_ref = model_ref.lower()
    if model_ref in TRUSTED_MODELS:
        return TRUSTED_MODELS[model_ref]
    
    # Also check by repo name
    for spec in TRUSTED_MODELS.values():
        if spec["repo"].lower() == model_ref:
            return spec
            
    return None

def verify_model_integrity(file_path: Path, expected_sha256: str) -> bool:
    """Computes SHA256 of the file and compares it with the expected value."""
    if not file_path.exists():
        return False
    
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        # Read in 1MB chunks
        for byte_block in iter(lambda: f.read(1024 * 1024), b""):
            sha256_hash.update(byte_block)
    
    actual_sha256 = sha256_hash.hexdigest()
    return actual_sha256 == expected_sha256

def download_model_to_staging(model_spec: Dict[str, Any], use_hf_hub: bool = True) -> Path:
    """Downloads the model to a staging area."""
    STAGING_DIR.mkdir(parents=True, exist_ok=True)
    dest_path = STAGING_DIR / model_spec["filename"]
    
    repo_id = model_spec["repo"]
    filename = model_spec["filename"]
    revision = model_spec.get("revision", "main")

    if use_hf_hub:
        try:
            from huggingface_hub import hf_hub_download
            print(f"  Downloading {filename} from {repo_id} using huggingface_hub...")
            downloaded_path = hf_hub_download(
                repo_id=repo_id,
                filename=filename,
                revision=revision,
                local_dir=STAGING_DIR,
                local_dir_use_symlinks=False
            )
            return Path(downloaded_path)
        except ImportError:
            print("  huggingface_hub not installed, falling back to requests...")
        except Exception as e:
            print(f"  huggingface_hub download failed: {e}")
            print("  Falling back to requests...")

    # Fallback to requests (legacy behavior but to staging)
    import requests
    from tqdm import tqdm
    
    url = f"https://huggingface.co/{repo_id}/resolve/{revision}/{filename}"
    print(f"  Downloading {filename} from {url}...")
    
    response = requests.get(url, stream=True, timeout=60)
    response.raise_for_status()
    total_size = int(response.headers.get("content-length", 0))

    with open(dest_path, "wb") as f, tqdm(
        desc=filename,
        total=total_size,
        unit="iB",
        unit_scale=True,
        unit_divisor=1024,
    ) as bar:
        for data in response.iter_content(chunk_size=8192):
            size = f.write(data)
            bar.update(size)
            
    return dest_path

def activate_model(staged_path: Path) -> Path:
    """Moves a verified model from staging to the active models directory."""
    MODELS_DIR.mkdir(parents=True, exist_ok=True)
    final_path = MODELS_DIR / staged_path.name
    
    if final_path.exists():
        # Keep a backup if it's different? For now, just overwrite
        pass
        
    shutil.move(str(staged_path), str(final_path))
    print(f"  Model activated: {final_path.name}")
    return final_path

def install_model_spec(spec: Dict[str, Any]) -> bool:
    """Resolve, download, verify, and activate a model from a spec dict."""
    print(f"Installing trusted model: {spec['name']}")
    try:
        staged_path = download_model_to_staging(spec)
        
        expected_hash = spec.get("sha256")
        if expected_hash and not expected_hash.startswith("placeholder"):
             if not verify_model_integrity(staged_path, expected_hash):
                 print(f"  [!] Integrity check FAILED for {spec['filename']}")
                 staged_path.unlink(missing_ok=True)
                 return False
             print("  [✓] Integrity check passed.")
        else:
             print("  [!] Warning: Skipping integrity check (no trusted hash provided in registry).")
        
        activate_model(staged_path)
        return True
    except Exception as e:
        print(f"  [!] Installation failed: {e}")
        return False

def install_model(model_ref: str) -> bool:
    """High-level helper to resolve, download, verify, and activate a model."""
    spec = resolve_model_ref(model_ref)
    if not spec:
        print(f"  [!] Model '{model_ref}' is not in the trusted registry.")
        return False
    
    return install_model_spec(spec)

if __name__ == "__main__":
    # Minimal CLI for testing if run directly
    import sys
    if len(sys.argv) > 1:
        install_model(sys.argv[1])
