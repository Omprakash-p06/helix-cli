#!/usr/bin/env python3
import argparse
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from huggingface_hub import HfApi, hf_hub_download
except ImportError:
    print("Missing dependency: huggingface_hub. Please run 'pip install huggingface_hub tqdm'")
    sys.exit(1)

try:
    from tqdm import tqdm
except ImportError:
    print("Missing dependency: tqdm. Please run 'pip install tqdm'")
    sys.exit(1)


PROJECT_DIR = Path(__file__).resolve().parent.parent
MODELS_DIR = PROJECT_DIR / "models"
STAGING_DIR = MODELS_DIR / ".staging"


def _scripts_dir() -> Path:
    return Path(__file__).resolve().parent


if str(_scripts_dir()) not in sys.path:
    sys.path.insert(0, str(_scripts_dir()))

from model_install import finalize_verified_download, resolve_model_ref


def normalize_repo_id(repo_id: str) -> str:
    cleaned = repo_id.strip().rstrip("/")
    if "huggingface.co/" in cleaned:
        cleaned = cleaned.split("huggingface.co/")[-1]
    for prefix in ("https://", "http://"):
        if cleaned.startswith(prefix):
            cleaned = cleaned.split(prefix, 1)[-1]
    if cleaned.startswith("hf.co/"):
        cleaned = cleaned.split("hf.co/")[-1]
    return cleaned.strip("/")


def format_size(size_bytes: int) -> str:
    if size_bytes <= 0:
        return "0 B"

    units = ("B", "KB", "MB", "GB", "TB")
    size = float(size_bytes)
    unit_index = 0
    while size >= 1024.0 and unit_index < len(units) - 1:
        size /= 1024.0
        unit_index += 1
    return f"{size:.2f} {units[unit_index]}"


def _api() -> HfApi:
    return HfApi()


def _repo_info(repo_id: str):
    return _api().repo_info(repo_id, files_metadata=True)


def _file_metadata(repo_info, filename: str) -> Optional[Dict[str, Any]]:
    for sibling in getattr(repo_info, "siblings", []):
        if sibling.rfilename == filename:
            lfs = getattr(sibling, "lfs", None)
            return {
                "filename": sibling.rfilename,
                "size": int(getattr(sibling, "size", 0) or 0),
                "sha256": getattr(lfs, "sha256", None),
                "repo_revision": getattr(repo_info, "sha", None),
            }
    return None


def list_repo_files(repo_id: str) -> List[Dict[str, Any]]:
    repo_id = normalize_repo_id(repo_id)
    repo_info = _repo_info(repo_id)

    gguf_files: List[Dict[str, Any]] = []
    for sibling in getattr(repo_info, "siblings", []):
        if sibling.rfilename.lower().endswith(".gguf"):
            lfs = getattr(sibling, "lfs", None)
            gguf_files.append(
                {
                    "repo_id": repo_id,
                    "filename": sibling.rfilename,
                    "size": int(getattr(sibling, "size", 0) or 0),
                    "sha256": getattr(lfs, "sha256", None),
                    "repo_revision": getattr(repo_info, "sha", None),
                }
            )

    if not gguf_files:
        raise ValueError(f"No .gguf files found in {repo_id}")

    return gguf_files


def list_repos_by_tag(tag: str, limit: int = 10) -> List[Dict[str, Any]]:
    results: List[Dict[str, Any]] = []
    for model in _api().list_models(search=tag, limit=limit, sort="downloads", direction=-1):
        results.append(
            {
                "repo_id": model.modelId,
                "downloads": getattr(model, "downloads", None),
                "likes": getattr(model, "likes", None),
                "tags": list(getattr(model, "tags", []) or []),
            }
        )
    return results


def download_file(repo_id: str, filename: str, revision: Optional[str] = None) -> Path:
    repo_id = normalize_repo_id(repo_id)
    repo_info = _repo_info(repo_id)
    metadata = _file_metadata(repo_info, filename)
    if metadata is None:
        raise ValueError(f"{filename} was not found in {repo_id}")

    expected_sha = metadata.get("sha256")
    if not expected_sha:
        raise ValueError(f"No SHA256 metadata available for {filename} in {repo_id}")

    revision = revision or metadata.get("repo_revision") or "main"
    local_dir = STAGING_DIR / repo_id.replace("/", "__")
    local_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n  Downloading {filename}")
    print(f"  Repo: {repo_id}")
    print(f"  Revision: {revision}")
    print(f"  Size: {format_size(metadata['size'])}")

    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename=filename,
        revision=revision,
        local_dir=str(local_dir),
        local_dir_use_symlinks=False,
    )

    staged_path = Path(downloaded_path)
    if not staged_path.exists():
        raise FileNotFoundError(f"Download completed but file not found: {staged_path}")

    print("  Verifying SHA256...")
    final_path = finalize_verified_download(staged_path, expected_sha)
    print(f"  [✓] Installed to {final_path}")
    return final_path


def mutate_config(filename: str) -> None:
    print(f"  [✓] Model '{filename}' is ready in models/.")
    print("  [✓] Next time you run `python start.py`, the launcher will discover it automatically.")


def _print_repo_files(files: List[Dict[str, Any]]) -> None:
    print("\n  Available GGUF files:")
    for index, file_obj in enumerate(files, 1):
        filename = file_obj["filename"]
        size_text = format_size(int(file_obj.get("size", 0) or 0))
        sha_prefix = (file_obj.get("sha256") or "")[0:8]
        sha_text = f" sha256:{sha_prefix}" if sha_prefix else ""
        print(f"  {index}) {filename}  ({size_text}){sha_text}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Download and install GGUF models from Hugging Face.")
    parser.add_argument("repo", nargs="?", help="Hugging Face repo ID or URL")
    parser.add_argument("--tag", help="Search popular repos by tag before downloading")
    args = parser.parse_args()

    repo_input = args.repo or input("\n  Enter Hugging Face Repo ID or URL (e.g. 'Bartowski/Meta-Llama-3-8B-Instruct-GGUF'): ")
    repo_id = normalize_repo_id(repo_input)

    if args.tag:
        print(f"\n  Top matching repos for tag '{args.tag}':")
        for entry in list_repos_by_tag(args.tag):
            print(f"  - {entry['repo_id']} (downloads={entry.get('downloads')}, likes={entry.get('likes')})")

    try:
        gguf_files = list_repo_files(repo_id)
    except Exception as exc:
        print(f"  [!] Failed to inspect repo: {exc}")
        return 1

    _print_repo_files(gguf_files)

    choice = 0
    while not (1 <= choice <= len(gguf_files)):
        try:
            choice = int(input(f"\n  Select file to download (1-{len(gguf_files)}): "))
        except ValueError:
            choice = 0

    selected = gguf_files[choice - 1]

    try:
        final_path = download_file(repo_id, selected["filename"])
        mutate_config(selected["filename"])
        print(f"\n  Installed model: {final_path}")
        print("\n" + "=" * 55)
        print("  Wizard Complete. You can now run `python start.py`.")
        print("=" * 55)
        return 0
    except KeyboardInterrupt:
        print("\n  [!] Download cancelled by user.")
        return 1
    except Exception as exc:
        print(f"\n  [!] Download failed: {exc}")
        return 1


if __name__ == "__main__":
    sys.exit(main())