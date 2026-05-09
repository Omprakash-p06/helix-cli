import json
import os
from pathlib import Path
from typing import Any, Dict, List


def _home_dir() -> Path:
    home = os.environ.get("HOME") or os.environ.get("USERPROFILE")
    if home:
        return Path(home)
    return Path(".")


def profile_path() -> Path:
    override = os.environ.get("HELIX_PROFILE_PATH")
    if override:
        return Path(override)
    return _home_dir() / ".helix" / "onboarding_profile.json"


def session_dir() -> Path:
    override = os.environ.get("HELIX_SESSION_DIR")
    if override:
        return Path(override)
    return _home_dir() / ".helix" / "sessions"


def latest_session_path() -> Path:
    return session_dir() / "session.latest.json"


def load_profile() -> Dict[str, Any]:
    path = profile_path()
    if not path.exists():
        return {}

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        if isinstance(data, dict):
            return data
    except Exception:
        pass

    return {}


def save_profile(profile: Dict[str, Any]) -> None:
    path = profile_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(profile, indent=2), encoding="utf-8")


def is_first_run(profile: Dict[str, Any]) -> bool:
    return not bool(profile.get("onboarding_complete"))


def resolve_default_model(profile: Dict[str, Any], models: List[Any]) -> str:
    preferred = str(profile.get("preferred_model", "")).strip()
    model_names = [m["name"] if isinstance(m, dict) else str(m) for m in models]
    if preferred and preferred in model_names:
        return preferred
    return model_names[0] if model_names else ""


def resolve_default_interface(profile: Dict[str, Any]) -> str:
    value = str(profile.get("preferred_interface", "tui")).strip().lower()
    return "web" if value == "web" else "tui"


def resolve_default_exec_mode(profile: Dict[str, Any]) -> str:
    value = str(profile.get("preferred_exec_mode", "agentic")).strip().lower()
    return "chat" if value == "chat" else "agentic"


def has_latest_session() -> bool:
    return latest_session_path().exists()


def update_profile(profile: Dict[str, Any], model: str, interface: str, exec_mode: str) -> Dict[str, Any]:
    profile["onboarding_complete"] = True
    if model.endswith(".gguf"):
        model = model[:-5]
    profile["preferred_model"] = model
    profile["preferred_interface"] = interface
    profile["preferred_exec_mode"] = exec_mode
    profile["last_updated"] = int(__import__("time").time())
    return profile
