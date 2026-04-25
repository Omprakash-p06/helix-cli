import sys
from pathlib import Path


PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import config
import system_check


def test_quantization_advice_reflects_configured_qwen_profile():
    advice = system_check.quantization_advice()

    assert config.MODEL_NAME in advice["detail"]
    assert config.MODEL_QUANTIZATION in advice["detail"]
    assert str(config.GPU_LAYERS) in advice["detail"]
    assert config.MODEL_SELECTION_GUIDANCE in advice["detail"]


def test_render_report_uses_config_driven_model_identity():
    report = system_check.render_report(
        {
            "Docker": {"status": "GREEN", "summary": "ok", "detail": "ok"},
            "Model": {"status": "GREEN", "summary": "ok", "detail": "ok"},
            "llama.cpp": {"status": "GREEN", "summary": "ok", "detail": "ok"},
            "Quantization": {"status": "GREEN", "summary": "ok", "detail": "ok"},
        }
    )

    assert f"Default model: {config.MODEL_NAME}" in report
    assert f"Default artifact: {Path(config.MODEL_PATH).name}" in report
    assert f"Backend hint: {config.BACKEND_HINT}" in report