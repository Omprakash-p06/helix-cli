from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts import onboarding_profile as op


def test_first_run_profile_creation(tmp_path: Path, monkeypatch):
    profile_file = tmp_path / "onboarding_profile.json"
    monkeypatch.setenv("HELIX_PROFILE_PATH", str(profile_file))

    profile = op.load_profile()
    assert profile == {}
    assert op.is_first_run(profile)

    updated = op.update_profile(profile, "model.gguf", "tui", "agentic")
    op.save_profile(updated)

    loaded = op.load_profile()
    assert loaded["onboarding_complete"] is True
    assert loaded["preferred_model"] == "model.gguf"
    assert loaded["preferred_interface"] == "tui"
    assert loaded["preferred_exec_mode"] == "agentic"


def test_returning_user_defaults_are_resolved(tmp_path: Path, monkeypatch):
    profile_file = tmp_path / "onboarding_profile.json"
    monkeypatch.setenv("HELIX_PROFILE_PATH", str(profile_file))

    profile = {
        "onboarding_complete": True,
        "preferred_model": "m2.gguf",
        "preferred_interface": "web",
        "preferred_exec_mode": "chat",
    }
    op.save_profile(profile)

    loaded = op.load_profile()
    models = ["m1.gguf", "m2.gguf"]
    assert not op.is_first_run(loaded)
    assert op.resolve_default_model(loaded, models) == "m2.gguf"
    assert op.resolve_default_interface(loaded) == "web"
    assert op.resolve_default_exec_mode(loaded) == "chat"


def test_has_latest_session_uses_session_dir(tmp_path: Path, monkeypatch):
    session_dir = tmp_path / "sessions"
    monkeypatch.setenv("HELIX_SESSION_DIR", str(session_dir))

    assert op.has_latest_session() is False

    session_dir.mkdir(parents=True, exist_ok=True)
    (session_dir / "session.latest.json").write_text("{}", encoding="utf-8")
    assert op.has_latest_session() is True
