#!/usr/bin/env python3
"""
System Check Module — Cross-platform hardware detection and performance rating.
Detects CPU (architecture, features, vendor), RAM, GPU (discrete + integrated),
rates hardware using an 8-factor scoring system, and recommends an optimal LLM
server configuration targeting ≥10 tok/s.
"""
import os
import platform
import subprocess
import re


# ─── Tier → Config Mapping (targeting ≥10 tok/s) ───────────────────────────

TIER_CONFIGS = {
    1: {
        "tier_name": "Minimal",
        "recommended_model": "Qwen3.5-9B-Uncensored",
        "recommended_file": "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf",
        "gpu_layers": 0,
        "context_size": 2048,
        "threads": 4,
        "batch_size": 256,
        "ubatch_size": 128,
    },
    2: {
        "tier_name": "Low",
        "recommended_model": "Qwen3.5-9B-Uncensored",
        "recommended_file": "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf",
        "gpu_layers": 0,
        "context_size": 4096,
        "threads": 6,
        "batch_size": 512,
        "ubatch_size": 256,
    },
    3: {
        "tier_name": "Mid",
        "recommended_model": "Qwen3.5-9B-Uncensored",
        "recommended_file": "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf",
        "gpu_layers": 33,
        "context_size": 8192,
        "threads": 6,
        "batch_size": 512,
        "ubatch_size": 256,
    },
    4: {
        "tier_name": "High",
        "recommended_model": "GPT-OSS-20B",
        "recommended_file": "gpt-oss-20b-IQ4_NL.gguf",
        "gpu_layers": 35,
        "context_size": 8192,
        "threads": 8,
        "batch_size": 512,
        "ubatch_size": 256,
    },
    5: {
        "tier_name": "Ultra",
        "recommended_model": "GPT-OSS-20B",
        "recommended_file": "gpt-oss-20b-IQ4_NL.gguf",
        "gpu_layers": -1,
        "context_size": 16384,
        "threads": 8,
        "batch_size": 1024,
        "ubatch_size": 512,
    },
}


# ─── CPU Detection ──────────────────────────────────────────────────────────

def detect_cpu():
    """Detect CPU model, core count, vendor, and instruction set features."""
    info = {
        "model": "Unknown",
        "vendor": "unknown",  # "intel", "amd", or "other"
        "cores_physical": 0,
        "cores_logical": 0,
        "features": [],       # e.g. ["avx2", "avx512f", "amx_int8"]
    }

    os_name = platform.system()

    try:
        info["cores_logical"] = os.cpu_count() or 0
    except Exception:
        pass

    if os_name == "Linux":
        try:
            with open("/proc/cpuinfo") as f:
                cpuinfo = f.read()

            # Model name
            match = re.search(r"model name\s*:\s*(.+)", cpuinfo)
            if match:
                info["model"] = match.group(1).strip()

            # Physical cores
            core_ids = set(re.findall(r"core id\s*:\s*(\d+)", cpuinfo))
            phys_ids = set(re.findall(r"physical id\s*:\s*(\d+)", cpuinfo))
            if core_ids and phys_ids:
                info["cores_physical"] = len(core_ids) * len(phys_ids)
            else:
                info["cores_physical"] = info["cores_logical"] // 2 or info["cores_logical"]

            # Vendor
            vendor_match = re.search(r"vendor_id\s*:\s*(.+)", cpuinfo)
            if vendor_match:
                vid = vendor_match.group(1).strip().lower()
                if "intel" in vid:
                    info["vendor"] = "intel"
                elif "amd" in vid:
                    info["vendor"] = "amd"

            # CPU features / flags
            flags_match = re.search(r"flags\s*:\s*(.+)", cpuinfo)
            if flags_match:
                all_flags = flags_match.group(1).strip().lower().split()
                relevant = {"avx", "avx2", "avx512f", "avx512bw", "avx512vl",
                             "avx512_vnni", "amx_int8", "amx_bf16", "sse4_1", "sse4_2", "f16c", "fma"}
                info["features"] = sorted(set(all_flags) & relevant)
        except Exception:
            info["cores_physical"] = info["cores_logical"] // 2 or info["cores_logical"]

    elif os_name == "Windows":
        try:
            result = subprocess.run(
                ["wmic", "cpu", "get", "Name,NumberOfCores,Manufacturer", "/format:csv"],
                capture_output=True, text=True, timeout=10
            )
            lines = [l.strip() for l in result.stdout.strip().split("\n") if l.strip()]
            if len(lines) >= 2:
                parts = lines[-1].split(",")
                if len(parts) >= 4:
                    mfr = parts[1].strip().lower()
                    info["model"] = parts[2].strip()
                    info["cores_physical"] = int(parts[3].strip())
                    if "intel" in mfr:
                        info["vendor"] = "intel"
                    elif "amd" in mfr or "advanced micro" in mfr:
                        info["vendor"] = "amd"
        except Exception:
            pass

        # Try to detect features on Windows via environment variable
        try:
            proc_id = os.environ.get("PROCESSOR_IDENTIFIER", "").lower()
            if "intel" in proc_id:
                info["vendor"] = "intel"
            elif "amd" in proc_id:
                info["vendor"] = "amd"
        except Exception:
            pass

        if not info["cores_physical"]:
            info["cores_physical"] = info["cores_logical"] // 2 or info["cores_logical"]

        model_lower = info["model"].lower()
        inferred = set(info["features"])
        if "ryzen" in model_lower:
            inferred.update(["sse4_2", "f16c", "fma", "avx", "avx2"])
        elif "intel" in model_lower and any(k in model_lower for k in ["core", "xeon"]):
            inferred.update(["sse4_2", "f16c", "fma", "avx"])
        info["features"] = sorted(inferred)
    else:
        info["cores_physical"] = info["cores_logical"] // 2 or info["cores_logical"]

    # Infer vendor from model name as fallback
    if info["vendor"] == "unknown":
        model_lower = info["model"].lower()
        if "intel" in model_lower:
            info["vendor"] = "intel"
        elif "amd" in model_lower or "ryzen" in model_lower:
            info["vendor"] = "amd"

    return info


# ─── RAM Detection ──────────────────────────────────────────────────────────

def detect_ram():
    """Detect total and available RAM in GB, and infer DDR type."""
    info = {"total_gb": 0.0, "available_gb": 0.0, "ddr_type": "unknown"}
    os_name = platform.system()

    if os_name == "Linux":
        try:
            with open("/proc/meminfo") as f:
                meminfo = f.read()
            total_match = re.search(r"MemTotal:\s+(\d+)\s+kB", meminfo)
            avail_match = re.search(r"MemAvailable:\s+(\d+)\s+kB", meminfo)
            if total_match:
                info["total_gb"] = round(int(total_match.group(1)) / 1024 / 1024, 1)
            if avail_match:
                info["available_gb"] = round(int(avail_match.group(1)) / 1024 / 1024, 1)
        except Exception:
            pass

        # Try to detect DDR type via dmidecode (works if run as root/admin)
        try:
            result = subprocess.run(
                ["dmidecode", "-t", "memory"],
                capture_output=True, text=True, timeout=2
            )
            if "DDR5" in result.stdout:
                info["ddr_type"] = "DDR5"
            elif "DDR4" in result.stdout:
                info["ddr_type"] = "DDR4"
            elif "DDR3" in result.stdout:
                info["ddr_type"] = "DDR3"
        except Exception:
            pass

    elif os_name == "Windows":
        try:
            result = subprocess.run(
                ["wmic", "ComputerSystem", "get", "TotalPhysicalMemory", "/format:csv"],
                capture_output=True, text=True, timeout=10
            )
            lines = [l.strip() for l in result.stdout.strip().split("\n") if l.strip()]
            if len(lines) >= 2:
                parts = lines[-1].split(",")
                if len(parts) >= 2:
                    info["total_gb"] = round(int(parts[-1]) / 1024 / 1024 / 1024, 1)

            result = subprocess.run(
                ["wmic", "OS", "get", "FreePhysicalMemory", "/format:csv"],
                capture_output=True, text=True, timeout=10
            )
            lines = [l.strip() for l in result.stdout.strip().split("\n") if l.strip()]
            if len(lines) >= 2:
                parts = lines[-1].split(",")
                if len(parts) >= 2:
                    info["available_gb"] = round(int(parts[-1]) / 1024 / 1024, 1)

            # DDR type on Windows
            result = subprocess.run(
                ["wmic", "memorychip", "get", "SMBIOSMemoryType", "/format:csv"],
                capture_output=True, text=True, timeout=10
            )
            if "34" in result.stdout:
                info["ddr_type"] = "DDR5"
            elif "26" in result.stdout:
                info["ddr_type"] = "DDR4"
        except Exception:
            pass

    return info


# ─── GPU Detection ──────────────────────────────────────────────────────────

def detect_gpu():
    """Detect NVIDIA discrete GPU model and VRAM via nvidia-smi."""
    info = {"model": None, "vram_gb": 0.0}
    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=name,memory.total", "--format=csv,noheader,nounits"],
            capture_output=True, text=True, timeout=10
        )
        if result.returncode == 0 and result.stdout.strip():
            line = result.stdout.strip().split("\n")[0]
            parts = [p.strip() for p in line.split(",")]
            if len(parts) >= 2:
                info["model"] = parts[0]
                info["vram_gb"] = round(int(parts[1]) / 1024, 1)
    except (FileNotFoundError, Exception):
        pass
    return info


# ─── iGPU Detection ─────────────────────────────────────────────────────────

def detect_igpu():
    """Detect integrated GPU (Intel Iris/UHD or AMD Radeon Vega/RDNA)."""
    info = {"model": None, "vendor": None}
    os_name = platform.system()

    if os_name == "Linux":
        try:
            result = subprocess.run(
                ["lspci"], capture_output=True, text=True, timeout=10
            )
            if result.returncode == 0:
                for line in result.stdout.split("\n"):
                    line_lower = line.lower()
                    if "vga" in line_lower or "display" in line_lower or "3d" in line_lower:
                        if "intel" in line_lower and ("iris" in line_lower or "uhd" in line_lower or "hd graphics" in line_lower):
                            info["model"] = line.split(":")[-1].strip()
                            info["vendor"] = "intel"
                        elif "amd" in line_lower or "radeon" in line_lower:
                            # Check if it's an iGPU (Vega, RDNA integrated)
                            if any(k in line_lower for k in ["vega", "renoir", "cezanne", "barcelo",
                                                              "rembrandt", "phoenix", "hawk", "rdna"]):
                                info["model"] = line.split(":")[-1].strip()
                                info["vendor"] = "amd"
        except Exception:
            pass

    elif os_name == "Windows":
        try:
            result = subprocess.run(
                ["wmic", "path", "win32_VideoController", "get", "Name", "/format:csv"],
                capture_output=True, text=True, timeout=10
            )
            if result.returncode == 0:
                for line in result.stdout.split("\n"):
                    line = line.strip()
                    if not line or line.lower().startswith("node,"):
                        continue
                    line_lower = line.lower()
                    if "intel" in line_lower and ("iris" in line_lower or "uhd" in line_lower or "hd graphics" in line_lower):
                        info["model"] = line.split(",")[-1].strip()
                        info["vendor"] = "intel"
                    elif (
                        "radeon" in line_lower
                        and (
                            "vega" in line_lower
                            or "radeon graphics" in line_lower
                            or "radeon(tm) graphics" in line_lower
                            or "amd radeon graphics" in line_lower
                        )
                        and " rx " not in f" {line_lower} "
                    ):
                        info["model"] = line.split(",")[-1].strip()
                        info["vendor"] = "amd"
        except Exception:
            pass

    return info


# ─── 8-Factor Scoring System ───────────────────────────────────────────────

def _score_cpu_arch(features):
    """Score CPU architecture based on instruction set extensions (1–5)."""
    if "amx_int8" in features or "amx_bf16" in features:
        return 5  # Intel 4th Gen Xeon / Core Ultra — matrix acceleration
    if "avx512f" in features:
        return 4  # AVX-512 — strong vector performance
    if "avx2" in features and "fma" in features:
        return 3  # AVX2+FMA — good for LLM inference
    if "avx2" in features:
        return 2  # AVX2 only
    if "sse4_2" in features:
        return 1  # Basic SIMD
    return 1


def _score_cores(physical_cores):
    """Score CPU core count (1–5)."""
    if physical_cores >= 16:
        return 5
    if physical_cores >= 8:
        return 4
    if physical_cores >= 6:
        return 3
    if physical_cores >= 4:
        return 2
    return 1


def _score_ram_total(total_gb):
    """Score total RAM (1–5)."""
    if total_gb >= 64:
        return 5
    if total_gb >= 32:
        return 4
    if total_gb >= 16:
        return 3
    if total_gb >= 8:
        return 2
    return 1


def _score_ram_available(available_gb):
    """Score available RAM (1–5)."""
    if available_gb >= 32:
        return 5
    if available_gb >= 16:
        return 4
    if available_gb >= 10:
        return 3
    if available_gb >= 6:
        return 2
    return 1


def _score_gpu_vram(gpu):
    """Score discrete GPU VRAM (0–5). 0 = no GPU."""
    if gpu["model"] is None:
        return 0
    vram = gpu["vram_gb"]
    if vram >= 12:
        return 5
    if vram >= 8:
        return 4
    if vram >= 4:
        return 3
    if vram >= 2:
        return 2
    return 1


def _score_igpu(igpu):
    """Score integrated GPU availability (0–3)."""
    if igpu["model"] is None:
        return 0
    if igpu["vendor"] == "intel":
        return 2  # Intel iGPU + OpenVINO is well-supported
    if igpu["vendor"] == "amd":
        return 2  # AMD iGPU + Vulkan is decent
    return 1


def _score_memory_bandwidth(ddr_type):
    """Score memory bandwidth from DDR type (1–3)."""
    if ddr_type == "DDR5":
        return 3
    if ddr_type == "DDR4":
        return 2
    return 1


def _score_vendor_fit(vendor, has_dgpu, igpu):
    """Score CPU vendor ecosystem fit for LLM acceleration (1–5)."""
    if has_dgpu:
        return 4  # Any vendor + dGPU is great
    if vendor == "intel" and igpu["vendor"] == "intel":
        return 3  # Intel CPU + Intel iGPU + OpenVINO
    if vendor == "amd" and igpu["vendor"] == "amd":
        return 3  # AMD CPU + AMD iGPU + Vulkan
    if vendor == "intel":
        return 2  # Intel CPU, no iGPU detected
    if vendor == "amd":
        return 2  # AMD CPU, no iGPU detected
    return 1


def rate_system(cpu, ram, gpu, igpu):
    """
    Rate system using 8-factor weighted scoring.
    Returns (tier, scores_dict, composite_score).
    """
    has_dgpu = gpu["model"] is not None

    scores = {
        "cpu_arch": _score_cpu_arch(cpu["features"]),
        "cpu_cores": _score_cores(cpu["cores_physical"] or cpu["cores_logical"]),
        "ram_total": _score_ram_total(ram["total_gb"]),
        "ram_available": _score_ram_available(ram["available_gb"]),
        "gpu_vram": _score_gpu_vram(gpu),
        "igpu": _score_igpu(igpu),
        "mem_bandwidth": _score_memory_bandwidth(ram["ddr_type"]),
        "vendor_fit": _score_vendor_fit(cpu["vendor"], has_dgpu, igpu),
    }

    # Weighted composite score
    weights = {
        "gpu_vram": 0.25,
        "cpu_arch": 0.15,
        "ram_total": 0.15,
        "vendor_fit": 0.10,
        "igpu": 0.10,
        "mem_bandwidth": 0.10,
        "cpu_cores": 0.10,
        "ram_available": 0.05,
    }

    composite = sum(scores[k] * weights[k] for k in weights)

    # Map composite score to tier
    if composite >= 4.5:
        tier = 5
    elif composite >= 3.5:
        tier = 4
    elif composite >= 2.5:
        tier = 3
    elif composite >= 1.5:
        tier = 2
    else:
        tier = 1

    # Cap tier at 2 if no GPU at all (neither discrete nor usable iGPU)
    if not has_dgpu and igpu["model"] is None:
        tier = min(tier, 2)

    return tier, scores, round(composite, 2)


# ─── Backend Recommendation ────────────────────────────────────────────────

def recommend_backend(cpu, gpu, igpu):
    """Recommend the best inference backend based on detected hardware."""
    has_dgpu = gpu["model"] is not None

    if has_dgpu:
        return "cuda"  # NVIDIA discrete GPU — use CUDA
    if igpu["vendor"] == "intel":
        return "openvino"  # Intel iGPU — use OpenVINO
    if igpu["vendor"] == "amd":
        return "vulkan"  # AMD iGPU — use Vulkan
    if cpu["vendor"] == "intel":
        return "openvino"  # Intel CPU without detected iGPU — still benefits from OpenVINO
    return "cpu"  # Pure CPU fallback


# ─── Recommended Config ────────────────────────────────────────────────────

def get_recommended_config(tier, gpu, igpu, backend_hint, ram=None):
    """Return a config dict based on tier, with backend-specific adjustments."""
    config = dict(TIER_CONFIGS[tier])
    config["backend_hint"] = backend_hint

    # ── Vulkan iGPU offload scaling (all tiers) ─────────────────────────────
    # The more layers offloaded to the iGPU, the higher the tok/s since the
    # compute runs on shader cores instead of the slower CPU FP32 units.
    if backend_hint == "vulkan" and igpu["model"] is not None:
        if tier == 1:
            config["gpu_layers"] = 8   # Very conservative for weak iGPUs
        elif tier == 2:
            config["gpu_layers"] = 16  # Partial offload — keeps RAM pressure low
        elif tier == 3:
            config["gpu_layers"] = 26  # Substantial offload for mid-tier AMD iGPUs
        elif tier == 4:
            config["gpu_layers"] = 35  # Near-full offload for strong Radeon 780M and above
        elif tier == 5:
            config["gpu_layers"] = 45  # Aggressive full offload

    # ── OpenVINO: acceleration managed by runtime, no -ngl needed ────────────
    if backend_hint == "openvino":
        config["gpu_layers"] = 0

    # ── CUDA: fine-tune GPU layers based on VRAM ─────────────────────────────
    if backend_hint == "cuda" and gpu["model"] is not None and config["gpu_layers"] > 0:
        vram = gpu["vram_gb"]
        if "9B" in config["recommended_model"] or "Qwen" in config["recommended_model"]:
            max_layers = int(vram / 0.3)
        else:
            max_layers = int(vram / 0.4)
        if config["gpu_layers"] != -1:
            config["gpu_layers"] = min(config["gpu_layers"], max_layers)

    # ── RAM Safety Guard ─────────────────────────────────────────────────────
    # Prevent model-load OOM freeze: if available RAM is critically low,
    # shrink context to reduce KV-cache memory footprint and preserve tok/s.
    if ram is not None:
        avail_gb = ram.get("available_gb", 16)
        if avail_gb < 4.0:
            # Critical: barely enough to load even a 9B Q4 model
            config["context_size"] = min(config["context_size"], 1024)
            config["gpu_layers"] = min(config.get("gpu_layers", 0), 4)
            config["batch_size"] = 256
            config["ubatch_size"] = 128
        elif avail_gb < 6.0:
            # Tight: reduce context aggressively to protect tok/s
            config["context_size"] = min(config["context_size"], 2048)
            config["batch_size"] = 256
            config["ubatch_size"] = 128

    return config


# ─── Full Detection ─────────────────────────────────────────────────────────

def detect_specs():
    """Run all detection and return a comprehensive specs dict."""
    cpu = detect_cpu()
    ram = detect_ram()
    gpu = detect_gpu()
    igpu = detect_igpu()
    tier, scores, composite = rate_system(cpu, ram, gpu, igpu)
    backend_hint = recommend_backend(cpu, gpu, igpu)
    config = get_recommended_config(tier, gpu, igpu, backend_hint, ram=ram)

    return {
        "cpu": cpu,
        "ram": ram,
        "gpu": gpu,
        "igpu": igpu,
        "tier": tier,
        "scores": scores,
        "composite": composite,
        "backend_hint": backend_hint,
        "config": config,
    }


# ─── Pretty Print ──────────────────────────────────────────────────────────

def print_specs(specs):
    """Print detected specs, scoring breakdown, and recommendation."""
    cpu = specs["cpu"]
    ram = specs["ram"]
    gpu = specs["gpu"]
    igpu = specs["igpu"]
    tier = specs["tier"]
    scores = specs["scores"]
    composite = specs["composite"]
    config = specs["config"]
    backend = specs["backend_hint"]

    print("=" * 55)
    print("  System Hardware Detection")
    print("=" * 55)

    # Hardware info
    print(f"\n  CPU:      {cpu['model']}")
    print(f"  Vendor:   {cpu['vendor'].upper()}")
    print(f"  Cores:    {cpu['cores_physical']} physical / {cpu['cores_logical']} logical")
    if cpu["features"]:
        print(f"  Features: {', '.join(cpu['features'])}")

    print(f"\n  RAM:      {ram['total_gb']} GB total / {ram['available_gb']} GB available")
    if ram["ddr_type"] != "unknown":
        print(f"  Type:     {ram['ddr_type']}")

    if gpu["model"]:
        print(f"\n  dGPU:     {gpu['model']} ({gpu['vram_gb']} GB VRAM)")
    else:
        print("\n  dGPU:     None")

    if igpu["model"]:
        print(f"  iGPU:     {igpu['model']}")
    else:
        print("  iGPU:     None detected")

    # Scoring breakdown
    print("\n" + "-" * 55)
    print("  Scoring Breakdown (8 factors)")
    print("-" * 55)
    labels = {
        "gpu_vram": ("GPU VRAM      ", 0.25),
        "cpu_arch": ("CPU Arch      ", 0.15),
        "ram_total": ("RAM Total     ", 0.15),
        "vendor_fit": ("Vendor Fit    ", 0.10),
        "igpu": ("iGPU          ", 0.10),
        "mem_bandwidth": ("Mem Bandwidth ", 0.10),
        "cpu_cores": ("CPU Cores     ", 0.10),
        "ram_available": ("RAM Available ", 0.05),
    }
    for key, (label, weight) in labels.items():
        score = scores[key]
        weighted = score * weight
        bar = "█" * score + "░" * (5 - score)
        print(f"  {label} [{bar}] {score}/5  (×{weight:.0%} = {weighted:.2f})")

    tier_name = config["tier_name"]
    tier_bar = "█" * tier + "░" * (5 - tier)
    print(f"\n  Composite:  {composite:.2f}")
    print(f"  Rating:     [{tier_bar}] Tier {tier}/5 — {tier_name}")
    print(f"  Backend:    {backend.upper()}")

    # Recommended config
    print("\n" + "-" * 55)
    print("  Recommended Configuration")
    print("-" * 55)
    print(f"  Model:        {config['recommended_model']}")
    print(f"  Backend:      {backend.upper()}")
    gl_label = "full offload" if config["gpu_layers"] == -1 else str(config["gpu_layers"])
    if backend == "openvino":
        gl_label = "N/A (OpenVINO managed)"
    print(f"  GPU Layers:   {gl_label}")
    print(f"  Context Size: {config['context_size']}")
    print(f"  Threads:      {config['threads']}")
    print(f"  Batch Size:   {config['batch_size']}")
    print("=" * 55)


# ─── Main (standalone) ─────────────────────────────────────────────────────

if __name__ == "__main__":
    specs = detect_specs()
    print_specs(specs)
