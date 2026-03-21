#!/usr/bin/env python3
"""
Helix Agent Setup - Phase 9 unified initialization.

What this installer does:
1) Detect hardware and recommend optimized config
2) Let user choose default GPT-OSS-20B, Qwen3.5-9B, or any HuggingFace GGUF
3) Install Python and Rust dependencies
4) Build llama.cpp with the detected backend flags
5) Run preflight checks: >=10 tok/s gate + agentic benchmark suite
6) Generate scripts/config.py with tuned values
"""

import os
import platform
import socket
import shutil
import subprocess
import sys
import time
import ctypes
from pathlib import Path


PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))

DEFAULT_MODELS = {
    "1": {
        "name": "GPT-OSS-20B",
        "repo": "DavidAU/OpenAi-GPT-oss-20b-abliterated-uncensored-NEO-Imatrix-gguf",
        "filename": "gpt-oss-20b-IQ4_NL.gguf",
    },
    "2": {
        "name": "Qwen3.5-9B-Uncensored",
        "repo": "HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive",
        "filename": "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf",
        "url": "https://huggingface.co/HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive/resolve/main/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf?download=true",
    },
}

KOBOLD_URLS = {
    "Windows": "https://github.com/LostRuins/koboldcpp/releases/latest/download/koboldcpp.exe",
    "Linux": "https://github.com/LostRuins/koboldcpp/releases/latest/download/koboldcpp-linux-x64",
    "Darwin": "https://github.com/LostRuins/koboldcpp/releases/latest/download/koboldcpp-mac-x64",
}

KOBOLD_FILENAMES = {
    "Windows": "koboldcpp.exe",
    "Linux": "koboldcpp-linux-x64",
    "Darwin": "koboldcpp-mac-x64",
}


def run_cmd(cmd, cwd=None, env=None):
    print(f"  $ {quote_cmd(cmd)}")
    subprocess.check_call(cmd, cwd=cwd, env=env)


def run_cmd_capture(cmd, cwd=None, env=None):
    print(f"  $ {quote_cmd(cmd)}")
    return subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True)


def quote_cmd(cmd):
    parts = []
    for part in cmd:
        if " " in part:
            parts.append(f'"{part}"')
        else:
            parts.append(part)
    return " ".join(parts)


def is_windows_admin():
    if platform.system() != "Windows":
        return False
    try:
        return bool(ctypes.windll.shell32.IsUserAnAdmin())
    except Exception:
        return False


def ensure_windows_admin_and_relaunch_if_needed():
    if platform.system() != "Windows":
        return
    if is_windows_admin():
        return

    print("[!] Administrator permission is required. Requesting UAC elevation...")
    try:
        script_path = os.path.abspath(__file__)
        relaunched_args = [sys.executable, script_path] + sys.argv[1:]
        command_line = subprocess.list2cmdline(relaunched_args)
        params = f'/k cd /d "{PROJECT_DIR}" && {command_line}'
        rc = ctypes.windll.shell32.ShellExecuteW(
            None,
            "runas",
            "cmd.exe",
            params,
            PROJECT_DIR,
            1,
        )
        if int(rc) <= 32:
            raise RuntimeError(f"ShellExecuteW returned {rc}")
    except Exception as exc:
        print(f"[!] Failed to relaunch with admin rights: {exc}")
        print("Run this script from an Administrator terminal and try again.")
        sys.exit(1)

    print("  Relaunched in an elevated window. Exiting current process.")
    sys.exit(0)


def install_python_dependencies():
    print("Checking Python dependencies...")
    packages = ["requests", "tqdm", "openai"]
    try:
        import requests  # noqa: F401
        import tqdm  # noqa: F401
    except Exception:
        print("Installing Python dependencies...")
        cmd = [sys.executable, "-m", "pip", "install"] + packages
        if platform.system() == "Linux":
            cmd.append("--break-system-packages")
        run_cmd(cmd)


ensure_windows_admin_and_relaunch_if_needed()
install_python_dependencies()
import requests
from tqdm import tqdm


def install_rust_toolchain_if_missing():
    if shutil.which("cargo"):
        return

    print("Rust toolchain not found. Installing rustup...")
    os_name = platform.system()
    try:
        if os_name in ("Linux", "Darwin"):
            run_cmd(["sh", "-c", "curl https://sh.rustup.rs -sSf | sh -s -- -y"])
            cargo_bin = os.path.expanduser("~/.cargo/bin")
            os.environ["PATH"] = cargo_bin + os.pathsep + os.environ.get("PATH", "")
        elif os_name == "Windows":
            if shutil.which("winget"):
                run_cmd(["winget", "install", "-e", "--id", "Rustlang.Rustup"])
            elif shutil.which("powershell"):
                rustup_ps = (
                    "$ErrorActionPreference='Stop';"
                    "$u='https://win.rustup.rs/x86_64';"
                    "$o=Join-Path $env:TEMP 'rustup-init.exe';"
                    "Invoke-WebRequest -Uri $u -OutFile $o;"
                    "& $o -y"
                )
                run_cmd(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", rustup_ps])
            else:
                raise RuntimeError("winget and powershell are unavailable on Windows")
        else:
            raise RuntimeError(f"Unsupported platform for automatic Rust install: {os_name}")
    except Exception as exc:
        print(f"[!] Failed to install Rust automatically: {exc}")
        print("Install Rust manually from https://rustup.rs and rerun setup.py")
        sys.exit(1)

    if not shutil.which("cargo"):
        print("[!] Rust installation completed but cargo was not found in PATH.")
        print("Open a new shell and run setup.py again.")
        sys.exit(1)


def find_vcvars64_bat():
    if platform.system() != "Windows":
        return None

    vswhere_candidates = []
    pf86 = os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)")
    vswhere_candidates.append(os.path.join(pf86, "Microsoft Visual Studio", "Installer", "vswhere.exe"))
    if shutil.which("vswhere"):
        vswhere_candidates.append(shutil.which("vswhere"))

    for vswhere in vswhere_candidates:
        if not vswhere or not os.path.exists(vswhere):
            continue
        try:
            result = run_cmd_capture(
                [
                    vswhere,
                    "-latest",
                    "-products",
                    "*",
                    "-requires",
                    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                    "-find",
                    r"VC\Auxiliary\Build\vcvars64.bat",
                ]
            )
            if result.returncode == 0:
                lines = result.stdout.strip().splitlines()
                if lines:
                    candidate = lines[-1].strip()
                    if os.path.exists(candidate):
                        return candidate
        except Exception:
            pass
    return None


def ensure_windows_cpp_build_tools():
    if platform.system() != "Windows":
        return None

    vcvars = find_vcvars64_bat()
    if vcvars:
        return vcvars

    print("Visual C++ Build Tools not detected. Attempting automatic install...")
    if not shutil.which("winget"):
        print("[!] winget is unavailable; cannot install Build Tools automatically.")
        print("Install Visual Studio Build Tools with C++ workload and rerun setup.py")
        sys.exit(1)

    run_cmd(
        [
            "winget",
            "install",
            "-e",
            "--id",
            "Microsoft.VisualStudio.2022.BuildTools",
            "--override",
            "--quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended",
        ]
    )
    vcvars = find_vcvars64_bat()
    if not vcvars:
        print("[!] Build Tools install completed but vcvars64.bat was not found.")
        print("Open a new shell and rerun setup.py.")
        sys.exit(1)
    return vcvars


def ensure_cmake_available():
    if shutil.which("cmake"):
        return

    print("cmake not found. Installing python cmake package...")
    try:
        cmd = [sys.executable, "-m", "pip", "install", "cmake"]
        if platform.system() == "Linux":
            cmd.append("--break-system-packages")
        run_cmd(cmd)
    except Exception as exc:
        print(f"[!] Failed to install cmake: {exc}")
        print("Install cmake manually and rerun setup.py")
        sys.exit(1)


def build_rust_agent():
    print("\nBuilding Rust orchestrator...")
    install_rust_toolchain_if_missing()
    agent_dir = os.path.join(PROJECT_DIR, "agent-rs")
    if platform.system() == "Windows":
        vcvars = ensure_windows_cpp_build_tools()
        if vcvars:
            launcher_path = os.path.join(agent_dir, "_build_rust_agent_temp.cmd")
            try:
                lines = [
                    "@echo off",
                    f'call "{vcvars}"',
                    "if errorlevel 1 exit /b %errorlevel%",
                    "cargo build",
                    "exit /b %errorlevel%",
                ]
                with open(launcher_path, "w", encoding="utf-8", newline="\r\n") as fh:
                    fh.write("\r\n".join(lines) + "\r\n")
                run_cmd(["cmd", "/d", "/c", launcher_path], cwd=agent_dir)
            finally:
                if os.path.exists(launcher_path):
                    os.remove(launcher_path)
            return
    run_cmd(["cargo", "build"], cwd=agent_dir)


def download_file(url, dest_path):
    if os.path.exists(dest_path):
        print(f"  Already exists: {os.path.basename(dest_path)} - skipping")
        return

    os.makedirs(os.path.dirname(dest_path), exist_ok=True)
    print(f"  Downloading {os.path.basename(dest_path)}...")
    try:
        response = requests.get(url, stream=True, timeout=60)
        response.raise_for_status()
        total_size = int(response.headers.get("content-length", 0))

        with open(dest_path, "wb") as file_handle, tqdm(
            desc=os.path.basename(dest_path),
            total=total_size,
            unit="iB",
            unit_scale=True,
            unit_divisor=1024,
        ) as bar:
            for data in response.iter_content(chunk_size=8192):
                written = file_handle.write(data)
                bar.update(written)
        print("  Download complete.")
    except Exception as exc:
        print(f"[!] Download failed: {exc}")
        if os.path.exists(dest_path):
            os.remove(dest_path)
        sys.exit(1)


def hf_repo_search(query):
    try:
        url = "https://huggingface.co/api/models"
        res = requests.get(url, params={"search": query, "limit": 10}, timeout=20)
        res.raise_for_status()
        items = res.json()
        repos = []
        for item in items:
            model_id = item.get("id", "")
            if model_id:
                repos.append(model_id)
        return repos
    except Exception:
        return []


def hf_tree_for_repo(repo_id):
    repo_id = repo_id.strip().rstrip("/")
    if "huggingface.co/" in repo_id:
        repo_id = repo_id.split("huggingface.co/")[-1]

    api_url = f"https://huggingface.co/api/models/{repo_id}/tree/main"
    res = requests.get(api_url, timeout=20)
    res.raise_for_status()
    tree = res.json()
    files = [f for f in tree if f.get("type") == "file" and f.get("path", "").endswith(".gguf")]
    return repo_id, files


def pick_universal_hf_model():
    print("\nUniversal HuggingFace mode")
    entry = input("  Enter HuggingFace repo URL/ID OR search term: ").strip()

    repo_id = entry
    if "/" not in entry and "huggingface.co" not in entry:
        matches = hf_repo_search(entry)
        if not matches:
            print("[!] No repositories found for that search term.")
            sys.exit(1)

        print("\n  Search results:")
        for idx, model_id in enumerate(matches, 1):
            print(f"  {idx}) {model_id}")
        choice = input(f"  Select repo (1-{len(matches)}): ").strip()
        try:
            repo_id = matches[int(choice) - 1]
        except Exception:
            print("[!] Invalid selection.")
            sys.exit(1)

    try:
        repo_id, gguf_files = hf_tree_for_repo(repo_id)
    except Exception as exc:
        print(f"[!] Failed to inspect HuggingFace repo: {exc}")
        sys.exit(1)

    if not gguf_files:
        print("[!] No .gguf files found in selected repo.")
        sys.exit(1)

    print("\n  GGUF files:")
    for idx, file_obj in enumerate(gguf_files, 1):
        path = file_obj.get("path", "")
        size = file_obj.get("size", 0)
        size_mb = round(size / (1024 * 1024), 1)
        print(f"  {idx}) {path} ({size_mb} MB)")

    file_choice = input(f"  Select file (1-{len(gguf_files)}): ").strip()
    try:
        selected = gguf_files[int(file_choice) - 1]
    except Exception:
        print("[!] Invalid file selection.")
        sys.exit(1)

    remote_path = selected["path"]
    filename = os.path.basename(remote_path)
    return {
        "name": f"{repo_id.split('/')[-1]}::{os.path.basename(filename)}",
        "repo": repo_id,
        "filename": filename,
        "remote_path": remote_path,
    }


def build_model_url(model_obj):
    if model_obj.get("url"):
        return model_obj["url"]
    remote_path = model_obj.get("remote_path", model_obj["filename"])
    return f"https://huggingface.co/{model_obj['repo']}/resolve/main/{remote_path}"


def find_nvcc_path():
    nvcc = shutil.which("nvcc")
    if nvcc:
        return nvcc
    if platform.system() == "Windows":
        cuda_root = Path(r"C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA")
        if cuda_root.exists():
            candidates = sorted(cuda_root.glob("v*/bin/nvcc.exe"), reverse=True)
            if candidates:
                return str(candidates[0])
    return None


def attempt_cuda_toolkit_install():
    if platform.system() != "Windows":
        return False
    if not shutil.which("winget"):
        return False

    package_ids = ["Nvidia.CUDA", "NVIDIA.CUDA"]
    for package_id in package_ids:
        try:
            run_cmd(
                [
                    "winget",
                    "install",
                    "-e",
                    "--id",
                    package_id,
                    "--accept-source-agreements",
                    "--accept-package-agreements",
                ]
            )
            break
        except Exception:
            continue

    nvcc = find_nvcc_path()
    if not nvcc:
        return False

    nvcc_bin = str(Path(nvcc).parent)
    os.environ["PATH"] = nvcc_bin + os.pathsep + os.environ.get("PATH", "")
    cuda_home = str(Path(nvcc).parent.parent)
    os.environ["CUDA_PATH"] = cuda_home
    os.environ["CUDA_HOME"] = cuda_home
    return True


def backend_is_available(backend_hint):
    if backend_hint == "cuda":
        return find_nvcc_path() is not None
    return True


def resolve_backend_hint(backend_hint):
    if backend_is_available(backend_hint):
        return backend_hint
    if backend_hint == "cuda":
        print("[!] CUDA backend was recommended, but nvcc was not found.")
        if attempt_cuda_toolkit_install() and backend_is_available("cuda"):
            print("    CUDA install succeeded; continuing with CUDA backend.")
            return "cuda"
        print("    Falling back to VULKAN if available, otherwise CPU.")
        return "vulkan"
    return "cpu"


def backend_fallback_chain(primary_backend):
    if primary_backend == "cuda":
        return ["cuda", "vulkan", "cpu"]
    if primary_backend == "vulkan":
        return ["vulkan", "cpu"]
    if primary_backend == "openvino":
        return ["openvino", "cpu"]
    return ["cpu"]


def clean_cmake_cache(build_dir):
    cache_path = os.path.join(build_dir, "CMakeCache.txt")
    files_dir = os.path.join(build_dir, "CMakeFiles")
    if os.path.exists(cache_path):
        os.remove(cache_path)
    if os.path.isdir(files_dir):
        shutil.rmtree(files_dir, ignore_errors=True)


def choose_models(auto_model_name):
    print("\nAvailable model setup choices:")
    for key, model in DEFAULT_MODELS.items():
        rec = " [recommended]" if model["name"] == auto_model_name else ""
        print(f"  {key}) {model['name']}{rec}")
    print("  3) Any HuggingFace GGUF (URL/repo/search)")
    print("  4) Download both default models")

    choice = input("\nEnter choice (1-4): ").strip()
    if choice in DEFAULT_MODELS:
        return [DEFAULT_MODELS[choice]]
    if choice == "3":
        return [pick_universal_hf_model()]
    if choice == "4":
        return [DEFAULT_MODELS["1"], DEFAULT_MODELS["2"]]

    print("[!] Invalid choice.")
    sys.exit(1)


def build_llama_cpp(backend_hint):
    print("\nBuilding llama.cpp with hardware backend...")
    ensure_cmake_available()

    src_dir = os.path.join(PROJECT_DIR, "llama.cpp")
    build_dir = os.path.join(src_dir, "build")
    os.makedirs(build_dir, exist_ok=True)

    attempted = []
    for candidate in backend_fallback_chain(backend_hint):
        attempted.append(candidate)
        flags = {
            "GGML_CUDA": "OFF",
            "GGML_VULKAN": "OFF",
            "GGML_OPENVINO": "OFF",
        }
        if candidate == "cuda":
            flags["GGML_CUDA"] = "ON"
        elif candidate == "vulkan":
            flags["GGML_VULKAN"] = "ON"
        elif candidate == "openvino":
            flags["GGML_OPENVINO"] = "ON"

        clean_cmake_cache(build_dir)
        print(f"  Trying backend: {candidate.upper()}")
        cmake_config = [
            "cmake",
            "-S",
            src_dir,
            "-B",
            build_dir,
            "-DCMAKE_BUILD_TYPE=Release",
        ] + [f"-D{k}={v}" for k, v in flags.items()]

        try:
            run_cmd(cmake_config)
            print("  Native compilation can take a while and may use 100% CPU on first build.")
            run_cmd(["cmake", "--build", build_dir, "--config", "Release", "-j"])
            return candidate
        except Exception as exc:
            print(f"  Backend {candidate.upper()} failed: {exc}")

    print(f"[!] Failed to build llama.cpp for all attempted backends: {', '.join(attempted)}")
    sys.exit(1)


def llm_cmd(llama_bin, model_path, context_size, cpu_threads, batch_size, ubatch_size, backend_hint, gpu_layers, port):
    cmd = [
        llama_bin,
        "-m",
        model_path,
        "-c",
        str(context_size),
        "-t",
        str(cpu_threads),
        "-b",
        str(batch_size),
        "-ub",
        str(ubatch_size),
        "--host",
        "127.0.0.1",
        "--port",
        str(port),
    ]
    if backend_hint in ("cuda", "vulkan") and gpu_layers != 0:
        cmd.extend(["-ngl", str(gpu_layers)])
    if backend_hint == "cuda" and gpu_layers != 0:
        cmd.extend(["-fa", "on"])
    if backend_hint in ("openvino", "vulkan", "cpu"):
        cmd.extend(["--no-mmap"])
    return cmd


def wait_for_server(url, retries=40):
    for _ in range(retries):
        try:
            res = requests.get(url, timeout=5)
            if res.status_code == 200:
                return True
        except Exception:
            pass
        time.sleep(1)
    return False


def tail_text_file(path, max_lines=30):
    if not path or not os.path.exists(path):
        return ""
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        lines = fh.readlines()
    return "".join(lines[-max_lines:]).strip()


def is_port_in_use(port):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(0.5)
        return sock.connect_ex(("127.0.0.1", port)) == 0


def pick_open_port(preferred_port, max_tries=30):
    for offset in range(max_tries):
        candidate = preferred_port + offset
        if not is_port_in_use(candidate):
            return candidate
    return preferred_port


def start_llama_server(cmd, tag):
    logs_dir = os.path.join(PROJECT_DIR, "logs")
    os.makedirs(logs_dir, exist_ok=True)
    timestamp = int(time.time())
    out_path = os.path.join(logs_dir, f"{tag}_{timestamp}.stdout.log")
    err_path = os.path.join(logs_dir, f"{tag}_{timestamp}.stderr.log")
    out_fh = open(out_path, "w", encoding="utf-8", errors="replace")
    err_fh = open(err_path, "w", encoding="utf-8", errors="replace")
    llama_dir = os.path.dirname(cmd[0]) if cmd else PROJECT_DIR
    env = os.environ.copy()
    env["PATH"] = llama_dir + os.pathsep + env.get("PATH", "")
    proc = subprocess.Popen(cmd, stdout=out_fh, stderr=err_fh, cwd=llama_dir, env=env)
    return proc, out_fh, err_fh, out_path, err_path


def wait_for_server_with_process(url, proc, retries=40):
    for _ in range(retries):
        if proc.poll() is not None:
            return False, f"process exited early with code {proc.returncode}"
        try:
            res = requests.get(url, timeout=3)
            if res.status_code == 200:
                return True, "ready"
        except Exception:
            pass
        time.sleep(1)
    return False, "server startup timed out"


def stop_process(proc):
    if not proc:
        return
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=20)
        except subprocess.TimeoutExpired:
            proc.kill()


def find_llama_server_binary():
    os_name = platform.system()
    bin_name = "llama-server.exe" if os_name == "Windows" else "llama-server"
    build_root = os.path.join(PROJECT_DIR, "llama.cpp", "build")
    if os_name == "Windows":
        candidates = [
            os.path.join(build_root, "bin", "Release", bin_name),
            os.path.join(build_root, "bin", bin_name),
            os.path.join(build_root, "bin", "Debug", bin_name),
            os.path.join(build_root, "Release", bin_name),
            os.path.join(build_root, "Debug", bin_name),
        ]
    else:
        candidates = [
            os.path.join(build_root, "bin", bin_name),
            os.path.join(build_root, "Release", bin_name),
            os.path.join(build_root, "Debug", bin_name),
        ]
    for path in candidates:
        if os.path.exists(path):
            return path
    if os.path.isdir(build_root):
        for root, _dirs, files in os.walk(build_root):
            if bin_name in files:
                return os.path.join(root, bin_name)
    return None


def stage_windows_runtime_dlls(llama_bin):
    if platform.system() != "Windows" or not llama_bin:
        return
    bin_dir = os.path.dirname(llama_bin)
    release_dir = os.path.join(os.path.dirname(bin_dir), "Release")
    if not os.path.isdir(release_dir):
        return
    dll_names = ["ggml-base.dll", "ggml-cpu.dll", "ggml-cuda.dll", "ggml.dll", "llama.dll", "mtmd.dll"]
    for dll in dll_names:
        src = os.path.join(release_dir, dll)
        dst = os.path.join(bin_dir, dll)
        if os.path.exists(src) and not os.path.exists(dst):
            shutil.copy2(src, dst)


def _measure_token_speed_once(llama_bin, model_path, selected_model_name, cpu_threads, batch_size, ubatch_size, backend_hint, gpu_layers, tag):
    benchmark_port = pick_open_port(8082)
    if benchmark_port != 8082:
        print(f"  Port 8082 in use; benchmark moved to port {benchmark_port}")

    cmd = llm_cmd(
        llama_bin,
        model_path,
        context_size=2048,
        cpu_threads=cpu_threads,
        batch_size=batch_size,
        ubatch_size=ubatch_size,
        backend_hint=backend_hint,
        gpu_layers=gpu_layers,
        port=benchmark_port,
    )

    proc = None
    out_fh = None
    err_fh = None
    out_log = ""
    err_log = ""
    try:
        proc, out_fh, err_fh, out_log, err_log = start_llama_server(cmd, tag)
        ok, reason = wait_for_server_with_process(f"http://127.0.0.1:{benchmark_port}/v1/models", proc, retries=60)
        if not ok:
            tail = tail_text_file(err_log, max_lines=40) or tail_text_file(out_log, max_lines=40)
            raise RuntimeError(
                "benchmark server failed to start\n"
                f"Reason: {reason}\n"
                f"Command: {quote_cmd(cmd)}\n"
                f"Stdout log: {out_log}\n"
                f"Stderr log: {err_log}\n"
                f"Recent log output:\n{tail or '(no output)'}"
            )

        models_res = requests.get(f"http://127.0.0.1:{benchmark_port}/v1/models", timeout=15)
        models_res.raise_for_status()
        loaded_models = models_res.json().get("data", [])
        loaded_model_id = selected_model_name
        if loaded_models and loaded_models[0].get("id"):
            loaded_model_id = loaded_models[0]["id"]

        payload = {
            "model": loaded_model_id,
            "prompt": "Write exactly 80 words explaining why testing matters.",
            "max_tokens": 80,
            "temperature": 0.1,
        }
        start_ts = time.time()
        res = requests.post(f"http://127.0.0.1:{benchmark_port}/v1/completions", json=payload, timeout=120)
        res.raise_for_status()
        end_ts = time.time()
        data = res.json()

        usage = data.get("usage", {})
        completion_tokens = usage.get("completion_tokens", 0)
        if completion_tokens <= 0:
            raise RuntimeError("token count unavailable in benchmark response")

        return completion_tokens / max(end_ts - start_ts, 1e-6)
    finally:
        if out_fh:
            out_fh.flush()
            out_fh.close()
        if err_fh:
            err_fh.flush()
            err_fh.close()
        stop_process(proc)


def _cuda_candidate_gpu_layers(user_gpu_layers, gpu_vram_gb):
    if gpu_vram_gb >= 16:
        base = [-1, 48, 32, 24]
    elif gpu_vram_gb >= 8:
        base = [32, 24, 18, 12]
    elif gpu_vram_gb >= 6:
        base = [24, 18, 12, 8]
    elif gpu_vram_gb >= 4:
        base = [33, 27, 21, 15, 12, 8]
    else:
        base = [12, 8, 4, 0]

    if user_gpu_layers not in (0, None):
        if user_gpu_layers == -1:
            probe = [-1] + base
        else:
            probe = [
                int(user_gpu_layers),
                int(user_gpu_layers) + 6,
                int(user_gpu_layers) + 3,
                int(user_gpu_layers) - 3,
                int(user_gpu_layers) - 6,
            ] + base
            probe = [x for x in probe if x == -1 or (1 <= x <= 64)]
        base = probe

    # Always test these known-good midrange CUDA offload values once.
    base = [22, 23] + base

    seen = set()
    ordered = []
    for value in base:
        if value not in seen:
            seen.add(value)
            ordered.append(value)
    return ordered


def enforce_token_speed(llama_bin, model_path, selected_model_name, cpu_threads, batch_size, ubatch_size, backend_hint, gpu_layers, gpu_vram_gb=0.0):
    print("\n" + "-" * 55)
    print("Token speed benchmark gate")
    print("-" * 55)

    candidates = [gpu_layers]
    if backend_hint == "cuda":
        candidates = _cuda_candidate_gpu_layers(gpu_layers, gpu_vram_gb)
        print(f"  CUDA mode: testing gpu_layers candidates {candidates}")

    best_tok_s = -1.0
    best_layers = gpu_layers
    last_error = None
    for idx, candidate in enumerate(candidates, 1):
        try:
            print(f"  Benchmark attempt {idx}/{len(candidates)} with gpu_layers={candidate}")
            tok_s = _measure_token_speed_once(
                llama_bin=llama_bin,
                model_path=model_path,
                selected_model_name=selected_model_name,
                cpu_threads=cpu_threads,
                batch_size=batch_size,
                ubatch_size=ubatch_size,
                backend_hint=backend_hint,
                gpu_layers=candidate,
                tag=f"token_benchmark_ngl_{candidate}",
            )
            print(f"  Measured speed: {tok_s:.2f} tok/s")
            if tok_s > best_tok_s:
                best_tok_s = tok_s
                best_layers = candidate
        except Exception as exc:
            last_error = exc
            print(f"  Attempt with gpu_layers={candidate} failed: {exc}")

    if best_tok_s >= 0:
        if best_tok_s >= 10.0:
            print(f"  PASS: >= 10 tok/s threshold satisfied")
            print(f"  Selected best configuration: gpu_layers={best_layers} at {best_tok_s:.2f} tok/s")
            return best_layers
        print(f"[!] Setup blocked: best measured speed was {best_tok_s:.2f} tok/s at gpu_layers={best_layers}.")
    elif last_error:
        print(f"[!] Setup blocked: all benchmark attempts failed to start. Last error: {last_error}")
    else:
        print("[!] Setup blocked: benchmark failed unexpectedly.")
    sys.exit(1)


def run_agentic_benchmark_preflight(llama_bin, model_path, cpu_threads, batch_size, ubatch_size, backend_hint, gpu_layers):
    print("\n" + "-" * 55)
    print("Phase 7 agentic benchmark preflight")
    print("-" * 55)

    eval_path = os.path.join(PROJECT_DIR, "tests", "eval.py")
    if not os.path.exists(eval_path):
        print("[!] Missing tests/eval.py; cannot run Phase 7 benchmark")
        sys.exit(1)

    preflight_port = pick_open_port(8080)
    if preflight_port != 8080:
        print(f"  Port 8080 in use; Phase 7 preflight moved to port {preflight_port}")

    cmd = llm_cmd(
        llama_bin,
        model_path,
        context_size=2048,
        cpu_threads=cpu_threads,
        batch_size=batch_size,
        ubatch_size=ubatch_size,
        backend_hint=backend_hint,
        gpu_layers=gpu_layers,
        port=preflight_port,
    )

    proc = None
    out_fh = None
    err_fh = None
    out_log = ""
    err_log = ""
    try:
        proc, out_fh, err_fh, out_log, err_log = start_llama_server(cmd, "phase7_preflight")
        ok, reason = wait_for_server_with_process(f"http://127.0.0.1:{preflight_port}/v1/models", proc, retries=70)
        if not ok:
            tail = tail_text_file(err_log, max_lines=40) or tail_text_file(out_log, max_lines=40)
            print("[!] Local model endpoint failed to start for Phase 7 preflight")
            print(f"    Reason: {reason}")
            print(f"    Command: {quote_cmd(cmd)}")
            print(f"    Stdout log: {out_log}")
            print(f"    Stderr log: {err_log}")
            if tail:
                print("    Recent log output:")
                for line in tail.splitlines():
                    print(f"      {line}")
            sys.exit(1)

        eval_env = os.environ.copy()
        eval_bin_name = "agent-rs.exe" if platform.system() == "Windows" else "agent-rs"
        eval_env["AGENT_BIN"] = os.path.join(PROJECT_DIR, "agent-rs", "target", "debug", eval_bin_name)
        run_cmd([sys.executable, eval_path], cwd=PROJECT_DIR, env=eval_env)
        report = os.path.join(PROJECT_DIR, "tests", "benchmark_results.md")
        if not os.path.exists(report):
            print("[!] Phase 7 benchmark did not generate tests/benchmark_results.md")
            sys.exit(1)
        print("  Phase 7 benchmark completed")
    finally:
        if out_fh:
            out_fh.flush()
            out_fh.close()
        if err_fh:
            err_fh.flush()
            err_fh.close()
        stop_process(proc)


def build_koboldcpp_preset(specs, backend_hint, context_size, cpu_threads, batch_size):
    """Return (args, profile_name, reasons) for an auto-tuned KoboldCPP preset."""
    args = []
    tier = specs.get("tier", 2)
    gpu = specs.get("gpu", {}) or {}
    reasons = []

    if tier >= 4:
        profile_name = "High Throughput"
    elif tier == 3:
        profile_name = "Balanced"
    else:
        profile_name = "Conservative"

    reasons.append(f"tier={tier}")
    reasons.append(f"backend_hint={backend_hint}")

    # Keep fallback conservative on OpenVINO-oriented systems.
    if backend_hint == "openvino":
        args.extend(["--gpulayers", "0"])
        reasons.append("openvino fallback uses CPU-safe gpulayers=0")
    else:
        # Use config-managed gpulayers in start_server.py by default.
        # For very high-end dGPU systems, nudging with a larger BLAS batch helps throughput.
        if gpu.get("model") and tier >= 4:
            args.extend(["--blasbatchsize", str(max(512, batch_size))])
            reasons.append("high-tier dGPU detected, increased blasbatchsize")

    # Memory-safety and latency balance for low/mid tiers.
    if tier <= 2:
        # Smaller processing chunks are generally more stable on weaker systems.
        args.extend(["--blasbatchsize", "256"])
        reasons.append("low-tier memory safety preset")

    # Threads preset (still explicit in command, but this keeps fallback self-describing).
    if cpu_threads <= 4:
        args.extend(["--threads", str(cpu_threads)])
        reasons.append("low-thread host, explicit threads preset")

    # Keep smart context enabled via launcher command. Avoid forcing flags that may
    # differ across KoboldCPP builds.
    deduped = []
    i = 0
    while i < len(args):
        flag = args[i]
        value = args[i + 1] if i + 1 < len(args) else ""
        # Replace earlier duplicate key with latest value.
        existing_idx = None
        j = 0
        while j + 1 < len(deduped):
            if deduped[j] == flag:
                existing_idx = j
            j += 2
        if existing_idx is not None:
            deduped[existing_idx + 1] = value
        else:
            deduped.extend([flag, value])
        i += 2

    return " ".join(deduped), profile_name, reasons


def generate_config(selected_model, models_downloaded, kobold_filename, koboldcpp_args, tier, tier_name, backend_hint, gpu_layers, context_size, cpu_threads, batch_size, ubatch_size):
    print("\nGenerating scripts/config.py...")
    available = {
        m["name"]: f'os.path.join(PROJECT_DIR, "models", "{m["filename"]}")'
        for m in models_downloaded
    }

    config_lines = [
        "import os",
        "",
        "PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))",
        "",
        f'MODEL_NAME = "{selected_model["name"]}"',
        f'MODEL_PATH = os.path.join(PROJECT_DIR, "models", "{selected_model["filename"]}")',
        "",
        "AVAILABLE_MODELS = {",
    ]
    for name, path_expr in available.items():
        config_lines.append(f'    "{name}": {path_expr},')
    config_lines.extend(
        [
            "}",
            "",
            'SERVER_HOST = "127.0.0.1"',
            "SERVER_PORT = 8080",
            'BASE_URL = f"http://{SERVER_HOST}:{SERVER_PORT}/v1"',
            f'KOBOLD_BIN = "{kobold_filename}"',
            f'KOBOLDCPP_ARGS = "{koboldcpp_args}"',
            "",
            f"# Performance (Tier {tier}/5 - {tier_name})",
            f"GPU_LAYERS = {gpu_layers}",
            f"CONTEXT_SIZE = {context_size}",
            f"CPU_THREADS = {cpu_threads}",
            f"BATCH_SIZE = {batch_size}",
            f"UBATCH_SIZE = {ubatch_size}",
            f'BACKEND_HINT = "{backend_hint}"',
            "",
            "REQUIRE_CONFIRMATION = True",
            "LOG_COMMANDS = True",
            'LOG_DIR = os.path.join(PROJECT_DIR, "logs")',
            "os.makedirs(LOG_DIR, exist_ok=True)",
            "",
            'DANGEROUS_COMMANDS = ["rm", "mv", "chmod", "dd", "mkfs", "fdisk", "systemctl", "reboot", "shutdown"]',
            "",
        ]
    )

    config_path = os.path.join(PROJECT_DIR, "scripts", "config.py")
    with open(config_path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(config_lines))
    print("  scripts/config.py written")


def main():
    scripts_dir = os.path.join(PROJECT_DIR, "scripts")
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    try:
        from helix_branding import print_helix_logo

        print_helix_logo(animated=True, delay=0.015)
        print()
    except Exception:
        # Setup should continue even if branding fails to load.
        pass

    if len(sys.argv) > 1 and sys.argv[1] in ["-h", "--help"]:
        print("Usage: python setup.py [--offline-check]")
        print("  --offline-check  Verify config/model/backend artifacts only")
        sys.exit(0)

    if len(sys.argv) > 1 and sys.argv[1] == "--offline-check":
        print("Verifying offline readiness...")
        config_path = os.path.join(PROJECT_DIR, "scripts", "config.py")
        issues = []
        if not os.path.exists(config_path):
            issues.append("scripts/config.py not found")
        else:
            sys.path.insert(0, os.path.join(PROJECT_DIR, "scripts"))
            import config

            if not os.path.exists(config.MODEL_PATH):
                issues.append(f"Model missing: {config.MODEL_PATH}")

            llama_bin = find_llama_server_binary()
            if not llama_bin or not os.path.exists(llama_bin):
                issues.append("llama-server missing in llama.cpp/build (checked Release/Debug layouts)")

            kobold_rel = getattr(config, "KOBOLD_BIN", "")
            if kobold_rel:
                kobold_path = os.path.join(PROJECT_DIR, kobold_rel)
                if not os.path.exists(kobold_path):
                    issues.append(f"KoboldCPP missing: {kobold_path}")

        if issues:
            for issue in issues:
                print(f"  x {issue}")
            sys.exit(1)

        print("  OK: offline artifacts verified")
        sys.exit(0)

    os_name = platform.system()
    print(f"\nOS: {os_name}")
    print(f"Project directory: {PROJECT_DIR}")

    sys.path.insert(0, os.path.join(PROJECT_DIR, "scripts"))
    from system_check import detect_specs, print_specs

    specs = detect_specs()
    print_specs(specs)

    tier = specs["tier"]
    auto_config = specs["config"]
    backend_hint = resolve_backend_hint(specs["backend_hint"])
    gpu_vram_gb = float((specs.get("gpu") or {}).get("vram_gb", 0.0) or 0.0)

    models_dir = os.path.join(PROJECT_DIR, "models")
    logs_dir = os.path.join(PROJECT_DIR, "logs")
    os.makedirs(models_dir, exist_ok=True)
    os.makedirs(logs_dir, exist_ok=True)

    models_to_download = choose_models(auto_config["recommended_model"])
    for model in models_to_download:
        url = build_model_url(model)
        path = os.path.join(models_dir, model["filename"])
        download_file(url, path)

    if len(models_to_download) > 1:
        print("\nChoose default runtime model:")
        for idx, model in enumerate(models_to_download, 1):
            rec = " [recommended]" if model["name"] == auto_config["recommended_model"] else ""
            print(f"  {idx}) {model['name']}{rec}")
        picked = input(f"Select default model (1-{len(models_to_download)}): ").strip()
        try:
            selected_model = models_to_download[int(picked) - 1]
        except Exception:
            selected_model = models_to_download[0]
    else:
        selected_model = models_to_download[0]

    print(f"\nSelected model: {selected_model['name']}")
    print(f"Recommended backend: {backend_hint.upper()}")

    use_auto = input("Use recommended config values? (Y/n): ").strip().lower()
    if use_auto in ("", "y", "yes"):
        gpu_layers = auto_config["gpu_layers"]
        context_size = auto_config["context_size"]
        cpu_threads = auto_config["threads"]
        batch_size = auto_config["batch_size"]
        ubatch_size = auto_config["ubatch_size"]
    else:
        try:
            gpu_layers = int(input("GPU layers (0=CPU, -1=full offload): ").strip())
        except ValueError:
            gpu_layers = auto_config["gpu_layers"]
        try:
            context_size = int(input("Context size (2048/4096/8192/16384): ").strip())
        except ValueError:
            context_size = auto_config["context_size"]
        try:
            cpu_threads = int(input("CPU threads: ").strip())
        except ValueError:
            cpu_threads = auto_config["threads"]
        try:
            batch_size = int(input("Batch size (256/512/1024): ").strip())
        except ValueError:
            batch_size = auto_config["batch_size"]
        ubatch_size = max(1, batch_size // 2)

    kobold_filename = ""
    koboldcpp_args = ""
    if os_name in KOBOLD_URLS:
        kobold_filename = KOBOLD_FILENAMES[os_name]
        kobold_path = os.path.join(PROJECT_DIR, kobold_filename)
        print("\nDownloading KoboldCPP fallback binary...")
        download_file(KOBOLD_URLS[os_name], kobold_path)
        if os_name != "Windows" and os.path.exists(kobold_path):
            os.chmod(kobold_path, 0o755)
        koboldcpp_args, kobold_profile, kobold_reasons = build_koboldcpp_preset(
            specs=specs,
            backend_hint=backend_hint,
            context_size=context_size,
            cpu_threads=cpu_threads,
            batch_size=batch_size,
        )
        print("\n  KoboldCPP Auto Preset")
        print("  ---------------------")
        print(f"  Profile : {kobold_profile}")
        print(f"  Args    : {koboldcpp_args or '(none)'}")
        if kobold_reasons:
            print("  Why     :")
            for reason in kobold_reasons:
                print(f"    - {reason}")

    build_rust_agent()
    if backend_hint == "openvino":
        print("\nInstalling OpenVINO Python package for Intel backend...")
        openvino_cmd = [sys.executable, "-m", "pip", "install", "openvino"]
        if platform.system() == "Linux":
            openvino_cmd.append("--break-system-packages")
        run_cmd(openvino_cmd)
    backend_hint = build_llama_cpp(backend_hint)
    if backend_hint not in ("cuda", "vulkan") and gpu_layers != 0:
        print(f"  Adjusting GPU layers to 0 for backend {backend_hint.upper()}")
        gpu_layers = 0

    generate_config(
        selected_model=selected_model,
        models_downloaded=models_to_download,
        kobold_filename=kobold_filename,
        koboldcpp_args=koboldcpp_args,
        tier=tier,
        tier_name=auto_config["tier_name"],
        backend_hint=backend_hint,
        gpu_layers=gpu_layers,
        context_size=context_size,
        cpu_threads=cpu_threads,
        batch_size=batch_size,
        ubatch_size=ubatch_size,
    )

    llama_bin = find_llama_server_binary()
    if not llama_bin or not os.path.exists(llama_bin):
        print("[!] Build completed but llama-server is missing in expected Windows/Linux locations.")
        sys.exit(1)
    stage_windows_runtime_dlls(llama_bin)
    print(f"  Using llama-server binary: {llama_bin}")

    selected_model_path = os.path.join(models_dir, selected_model["filename"])
    gpu_layers = enforce_token_speed(
        llama_bin=llama_bin,
        model_path=selected_model_path,
        selected_model_name=selected_model["name"],
        cpu_threads=cpu_threads,
        batch_size=batch_size,
        ubatch_size=ubatch_size,
        backend_hint=backend_hint,
        gpu_layers=gpu_layers,
        gpu_vram_gb=gpu_vram_gb,
    )

    generate_config(
        selected_model=selected_model,
        models_downloaded=models_to_download,
        kobold_filename=kobold_filename,
        koboldcpp_args=koboldcpp_args,
        tier=tier,
        tier_name=auto_config["tier_name"],
        backend_hint=backend_hint,
        gpu_layers=gpu_layers,
        context_size=context_size,
        cpu_threads=cpu_threads,
        batch_size=batch_size,
        ubatch_size=ubatch_size,
    )

    run_agentic_benchmark_preflight(
        llama_bin=llama_bin,
        model_path=selected_model_path,
        cpu_threads=cpu_threads,
        batch_size=batch_size,
        ubatch_size=ubatch_size,
        backend_hint=backend_hint,
        gpu_layers=gpu_layers,
    )

    print("\n" + "=" * 60)
    print("Setup complete. System passed Phase 9 preflight requirements.")
    print("=" * 60)
    print("Start services with:")
    print("  Terminal 1: python scripts/start_server.py")
    print("  Terminal 2: cd agent-rs && cargo run")
    print("Offline check:")
    print("  python setup.py --offline-check")


if __name__ == "__main__":
    main()
