from pathlib import Path


ROOT = Path(__file__).parents[1]
GUIDE = ROOT / "docs" / "guia-servidor-supervisor-cliente-tls.md"
LAUNCHER = ROOT / "scripts" / "run_production_gate.ps1"


def test_windows_prerequisite_checks_are_documented():
    text = GUIDE.read_text(encoding="utf-8")
    for marker in (
        "wsl.exe --status",
        "wsl.exe -l -v",
        "docker.exe version",
        "docker.exe compose version",
    ):
        assert marker in text


def test_linux_prerequisite_checks_are_documented():
    text = GUIDE.read_text(encoding="utf-8")
    for marker in ("docker version", "docker compose version", "bash --version"):
        assert marker in text


def test_launcher_handles_internal_and_user_distributions():
    text = LAUNCHER.read_text(encoding="utf-8").lower()
    assert "docker-desktop" in text
    assert "docker-desktop-data" in text
    assert "no linux distribution is installed" in text
    assert "linuxdistribution" in text
