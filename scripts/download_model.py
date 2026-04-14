#!/usr/bin/env python3
import os
import sys
import re
import math
try:
    import requests
    from tqdm import tqdm
except ImportError:
    print("Missing dependencies. Please run 'pip install requests tqdm'")
    sys.exit(1)

PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))

def format_size(size_bytes):
    if size_bytes == 0:
        return "0B"
    size_name = ("B", "KB", "MB", "GB", "TB")
    i = int(math.floor(math.log(size_bytes, 1024)))
    p = math.pow(1024, i)
    s = round(size_bytes / p, 2)
    return f"{s} {size_name[i]}"

def get_gguf_files(repo_id):
    repo_id = repo_id.strip().rstrip("/")
    if "huggingface.co/" in repo_id:
        repo_id = repo_id.split("huggingface.co/")[-1]
        
    print(f"\n  Querying HuggingFace API for repo: {repo_id} ...")
    url = f"https://huggingface.co/api/models/{repo_id}/tree/main"
    
    try:
        res = requests.get(url, timeout=10)
        res.raise_for_status()
        tree = res.json()
    except Exception as e:
        print(f"  [!] Failed to query HuggingFace API: {e}")
        print("  Ensure the repo exists, is public, and is formatted as 'Author/Model'.")
        sys.exit(1)

    gguf_files = []
    for item in tree:
        if item.get("type") == "file" and item.get("path", "").endswith(".gguf"):
            gguf_files.append(item)
            
    if not gguf_files:
        print("  [!] No .gguf files found in the root of this repository.")
        sys.exit(1)
        
    return repo_id, gguf_files

def download_file(url, dest_path):
    if os.path.exists(dest_path):
        print(f"\n  Already exists: {os.path.basename(dest_path)} — skipping download.")
        return True

    print(f"\n  Downloading {os.path.basename(dest_path)}...")
    try:
        response = requests.get(url, stream=True)
        response.raise_for_status()
        total_size = int(response.headers.get("content-length", 0))

        with open(dest_path, "wb") as file, tqdm(
            desc=os.path.basename(dest_path),
            total=total_size,
            unit="iB",
            unit_scale=True,
            unit_divisor=1024,
        ) as bar:
            for data in response.iter_content(chunk_size=8192):
                size = file.write(data)
                bar.update(size)
        print("  Download complete.")
        return True
    except KeyboardInterrupt:
        print("\n  [!] Download cancelled by user.")
        if os.path.exists(dest_path):
            os.remove(dest_path)
        sys.exit(1)
    except Exception as e:
        print(f"  Error downloading: {e}")
        if os.path.exists(dest_path):
            os.remove(dest_path)
        sys.exit(1)

def mutate_config(filename):
    # In Phase 3, config.py natively sorts models by `os.path.getmtime`.
    # Therefore, simply placing a fresh `.gguf` into the models/ folder guarantees it runs next.
    print(f"  [✓] Model '{filename}' physically placed in models/ directory.")
    print("  [✓] Next time you run `python start.py`, the agent will automatically boot this new model natively.")

def main():
    print("=" * 55)
    print("  Universal HuggingFace Model Downloader")
    print("=" * 55)
    
    # Ensure scripts directory is in path for imports
    scripts_dir = os.path.dirname(os.path.abspath(__file__))
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
        
    from model_install import install_model_spec, resolve_model_ref

    # Simple check for CLI args to simulate --help
    if len(sys.argv) > 1 and sys.argv[1] in ["-h", "--help"]:
        print("\n  Usage: python download_model.py")
        print("  Follow the interactive prompts to download any GGUF from hf.co")
        sys.exit(0)
    
    repo_input = input("\n  Enter HuggingFace Repo ID or URL (e.g. 'Bartowski/Meta-Llama-3-8B-Instruct-GGUF'): ")
    repo_id, gguf_files = get_gguf_files(repo_input)
    
    print("\n  Available Quantizations:")
    for i, file_obj in enumerate(gguf_files, 1):
        filename = file_obj.get("path")
        size_bytes = file_obj.get("size", 0)
        print(f"  {i}) {filename}  ({format_size(size_bytes)})")
        
    choice = 0
    while not (1 <= choice <= len(gguf_files)):
        try:
            choice = int(input(f"\n  Select file to download (1-{len(gguf_files)}): "))
        except ValueError:
            pass
            
    selected_file = gguf_files[choice - 1]["path"]
    
    # Resolve against trusted registry if it exists
    model_spec = {
        "name": f"{repo_id.split('/')[-1]}::{selected_file}",
        "repo": repo_id,
        "filename": selected_file,
    }
    
    trusted = resolve_model_ref(repo_id)
    if trusted and trusted["filename"] == selected_file:
         model_spec.update(trusted)
         
    if install_model_spec(model_spec):
        mutate_config(selected_file)
        
    print("\n" + "=" * 55)
    print("  Wizard Complete. You can now run `python start.py`.")
    print("=" * 55)

if __name__ == "__main__":
    main()
