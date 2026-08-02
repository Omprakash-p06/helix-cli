import sys
from pathlib import Path

import pytest


PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import config


@pytest.mark.parametrize(
    "vram, expected_quantization, expected_backend_hint, expected_gpu_layers",
    [
        (0, "Q4_K_M", "cpu", 0),
        (8, "Q4_K_M", "cuda", 24),
        (12, "Q5_K_M", "cuda", 48),
        (24, "Q8_0", "cuda", -1),
    ],
)
def test_qwen_27b_variant_selection_by_vram(vram, expected_quantization, expected_backend_hint, expected_gpu_layers):
    entry = config.build_model_entry("Qwen-3.6-27B-MoE", vram)

    assert entry["quantization"] == expected_quantization
    assert entry["backend_hint"] == expected_backend_hint
    assert entry["gpu_layers"] == expected_gpu_layers


@pytest.mark.parametrize(
    "vram, expected_quantization, expected_backend_hint, expected_gpu_layers",
    [
        (0, "Q4_K_M", "cpu", 0),
        (16, "Q4_K_M", "cuda", 24),
        (24, "Q5_K_M", "cuda", 40),
        (32, "Q8_0", "cuda", -1),
    ],
)
def test_qwen_35b_variant_selection_by_vram(vram, expected_quantization, expected_backend_hint, expected_gpu_layers):
    entry = config.build_model_entry("Qwen-3.6-35B-MoE", vram)

    assert entry["quantization"] == expected_quantization
    assert entry["backend_hint"] == expected_backend_hint
    assert entry["gpu_layers"] == expected_gpu_layers


def test_qwen_catalog_exposes_expected_models():
    assert config.DEFAULT_MODEL_NAME == "Qwen-3.6-27B-MoE"
    assert set(config.MODEL_CATALOG) == {"Qwen-3.6-27B-MoE", "Qwen-3.6-35B-MoE", "Gemma-4-E4B"}
    assert config.AVAILABLE_MODELS["Qwen-3.6-27B-MoE"]["repo_alias"] == "qwen-3.6-27b-moe"
    assert config.AVAILABLE_MODELS["Qwen-3.6-35B-MoE"]["repo_alias"] == "qwen-3.6-35b-moe"


def test_gemma_4_e4b_catalog_entry():
    """Gemma-4-E4B exposes 3 VRAM-tiered variants with jinja + mmproj metadata."""
    assert "Gemma-4-E4B" in config.MODEL_CATALOG
    entry = config.MODEL_CATALOG["Gemma-4-E4B"]
    assert entry["repo_alias"] == "gemma-4-e4b"
    variants = entry["variants"]
    assert len(variants) == 3
    assert [v["min_vram_gb"] for v in variants] == [8, 4, 0]
    for variant in variants:
        assert variant["jinja"] is True
        assert variant["mmproj_filename"] == "mmproj-F16.gguf"


@pytest.mark.parametrize(
    "vram, expected_quantization, expected_backend_hint, expected_gpu_layers",
    [
        (0, "Q4_K_M", "cpu", 0),
        (4, "Q4_K_M", "cuda", -1),
        (8, "Q8_0", "cuda", -1),
        (24, "Q8_0", "cuda", -1),
    ],
)
def test_gemma_4_e4b_variant_selection_by_vram(vram, expected_quantization, expected_backend_hint, expected_gpu_layers):
    entry = config.build_model_entry("Gemma-4-E4B", vram)

    assert entry["quantization"] == expected_quantization
    assert entry["backend_hint"] == expected_backend_hint
    assert entry["gpu_layers"] == expected_gpu_layers