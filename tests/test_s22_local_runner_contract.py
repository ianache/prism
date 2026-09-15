from pathlib import Path

import pytest


ROOT = Path(__file__).parents[1]
LAUNCHER = ROOT / "scripts" / "run_production_gate.ps1"


def launcher_text() -> str:
    if not LAUNCHER.exists():
        pytest.fail("scripts/run_production_gate.ps1 is not implemented")
    return LAUNCHER.read_text(encoding="utf-8")


def test_launcher_declares_safe_defaults_and_protocol_set():
    text = launcher_text()
    assert "Frames" in text
    assert "100000" in text
    assert "BatchSize" in text
    assert "64" in text
    assert "ValidateSet('tcp', 'http')" in text or 'ValidateSet("tcp", "http")' in text
    assert "PRISM_ALLOW_CYCLE" in text
    assert "false" in text.lower()
    assert "docker_production_readiness.sh" in text


def test_launcher_rejects_invalid_numeric_values_and_secret_arguments():
    text = launcher_text()
    assert "Frames must be positive" in text
    assert "BatchSize must be positive" in text
    assert "token" in text.lower()
    assert "private key" in text.lower() or "pem" in text.lower()


def test_launcher_reports_missing_prerequisites_deterministically():
    text = launcher_text()
    assert "wsl.exe" in text.lower()
    assert "docker.exe" in text.lower()
    assert "bash" in text.lower()
    assert "distribution" in text.lower()
    assert "Write-Failure" in text


def test_launcher_uses_strict_mode_and_safe_wsl_invocation():
    text = launcher_text()
    assert "Set-StrictMode -Version Latest" in text
    assert "wslpath" in text
    assert "--distribution" in text
    assert "bash -lc" in text
    assert "--frames $Frames" in text
    assert "--batch-size $BatchSize" in text
    assert "--evidence-dir $evidenceArg" in text
    assert "PRISM_PROTOCOL" in text
    assert "Invoke-Expression" not in text
