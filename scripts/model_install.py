#!/usr/bin/env python3
import argparse
import hashlib
import os
import shutil
import sys
from pathlib import Path
from typing import Any, Dict, Optional


TRUSTED_MODELS: Dict[str, Dict[str, Any]] = {
    "qwen-3.6-27b-moe": {
        "name": "Qwen-3.6-27B-MoE",
        "repo": "Qwen/Qwen3.6-27B-Instruct-GGUF",
        "filename": "Qwen3.6-27B-Instruct-Q4_K_M.gguf",
        "quantizations": ["Q4_K_M", "Q5_K_M", "Q8_0"],
        "revision": "UNVERIFIED_REVISION",
        "sha256": None,
        "verification_status": "blocked-until-pinned",
    },
    "qwen-3.6-35b-moe": {
        "name": "Qwen-3.6-35B-MoE",
        "repo": "Qwen/Qwen3.6-35B-Instruct-GGUF",
        "filename": "Qwen3.6-35B-Instruct-Q4_K_M.gguf",
        "quantizations": ["Q4_K_M", "Q5_K_M", "Q8_0"],
        "revision": "UNVERIFIED_REVISION",
        "sha256": None,
        "verification_status": "blocked-until-pinned",
    },
}

PROJECT_DIR = Path(__file__).parent.parent.absolute()
MODELS_DIR = PROJECT_DIR / "models"
STAGING_DIR = MODELS_DIR / ".staging"


def resolve_model_ref(model_ref: str) -> Optional[Dict[str, Any]]:
    model_ref = model_ref.lower()
    if model_ref in TRUSTED_MODELS:
        return TRUSTED_MODELS[model_ref]

    for spec in TRUSTED_MODELS.values():
        if spec["repo"].lower() == model_ref or spec["name"].lower() == model_ref:
            return spec

    return None


def verify_model_integrity(file_path: Path, expected_sha256: str) -> bool:
    if not file_path.exists():
        return False

    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as handle:
        for byte_block in iter(lambda: handle.read(1024 * 1024), b""):
            sha256_hash.update(byte_block)

    return sha256_hash.hexdigest() == expected_sha256


def validate_trusted_model_spec(model_spec: Dict[str, Any]) -> None:
    revision = model_spec.get("revision")
    sha256 = model_spec.get("sha256")
    if not revision or revision == "main" or revision == "UNVERIFIED_REVISION":
        raise ValueError(
            f"{model_spec['name']} is blocked: pin a concrete Hugging Face revision before installation."
        )
    if not isinstance(sha256, str) or len(sha256) != 64:
        raise ValueError(
            f"{model_spec['name']} is blocked: add a verified 64-character SHA256 before installation."
        )


def download_model_to_staging(model_spec: Dict[str, Any], use_hf_hub: bool = True) -> Path:
    STAGING_DIR.mkdir(parents=True, exist_ok=True)
    dest_path = STAGING_DIR / model_spec["filename"]

    repo_id = model_spec["repo"]
    filename = model_spec["filename"]
    revision = model_spec["revision"]

    if use_hf_hub:
        try:
            from huggingface_hub import hf_hub_download

            print(f"  Downloading {filename} from {repo_id} @ {revision} using huggingface_hub...")
            downloaded_path = hf_hub_download(
                repo_id=repo_id,
                filename=filename,
                revision=revision,
                local_dir=STAGING_DIR,
                local_dir_use_symlinks=False,
            )
            return Path(downloaded_path)
        except ImportError:
            print("  huggingface_hub not installed, falling back to requests...")
        except Exception as exc:
            print(f"  huggingface_hub download failed: {exc}")
            print("  Falling back to requests...")

    import requests
    from tqdm import tqdm

    url = f"https://huggingface.co/{repo_id}/resolve/{revision}/{filename}"
    print(f"  Downloading {filename} from {url}...")

    response = requests.get(url, stream=True, timeout=60)
    response.raise_for_status()
    total_size = int(response.headers.get("content-length", 0))

    with open(dest_path, "wb") as handle, tqdm(
        desc=filename,
        total=total_size,
        unit="iB",
        unit_scale=True,
        unit_divisor=1024,
    ) as bar:
        for data in response.iter_content(chunk_size=8192):
            size = handle.write(data)
            bar.update(size)

    return dest_path


def activate_model(staged_path: Path) -> Path:
    MODELS_DIR.mkdir(parents=True, exist_ok=True)
    try:
        relative_path = staged_path.relative_to(STAGING_DIR)
    except ValueError:
        relative_path = Path(staged_path.name)

    final_path = MODELS_DIR / relative_path
    final_path.parent.mkdir(parents=True, exist_ok=True)
    if final_path.exists():
        final_path.unlink()
    shutil.move(str(staged_path), str(final_path))
    print(f"  Model activated: {final_path.name}")
    return final_path


def finalize_verified_download(staged_path: Path, expected_sha256: Optional[str] = None) -> Path:
    if expected_sha256:
        if not verify_model_integrity(staged_path, expected_sha256):
            print(f"  [!] Integrity check FAILED for {staged_path.name}")
            staged_path.unlink(missing_ok=True)
            raise ValueError(f"Integrity check failed for {staged_path.name}")
        print("  [✓] Integrity check passed.")

    return activate_model(staged_path)


def install_model_spec(spec: Dict[str, Any]) -> bool:
    print(f"Installing trusted model: {spec['name']}")
    try:
        validate_trusted_model_spec(spec)
        staged_path = download_model_to_staging(spec)
        finalize_verified_download(staged_path, spec["sha256"])
        return True
    except Exception as exc:
        print(f"  [!] Installation failed: {exc}")
        return False


def install_model(model_ref: str) -> bool:
    spec = resolve_model_ref(model_ref)
    if not spec:
        print(f"  [!] Model '{model_ref}' is not in the trusted registry.")
        return False

    return install_model_spec(spec)


def list_models() -> int:
    print("Trusted Models Registry:")
    print("-" * 72)
    for alias, spec in TRUSTED_MODELS.items():
        quants = ", ".join(spec.get("quantizations", []))
        print(f"{alias:18} | {spec['name']}")
        print(f"{'':18} | Repo: {spec['repo']}")
        print(f"{'':18} | Revision: {spec['revision']}")
        print(f"{'':18} | Quantizations: {quants}")
        print(f"{'':18} | Status: {spec.get('verification_status', 'verified')}")
        print("-" * 72)
    return 0


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Install or inspect trusted Helix models.")
    parser.add_argument("model", nargs="?", help="Model alias or repo ID to install")
    parser.add_argument("--list-models", action="store_true", help="List trusted models and exit")
    args = parser.parse_args(argv)

    if args.list_models:
        return list_models()

    if not args.model:
        parser.print_help()
        return 1

    return 0 if install_model(args.model) else 1


if __name__ == "__main__":
    sys.exit(main())
