import subprocess
from pathlib import Path


ROOT = Path(__file__).parents[1]
LAUNCHER = ROOT / "scripts" / "run_production_gate.ps1"


def run_launcher(*arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "powershell.exe",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            str(LAUNCHER),
            *arguments,
        ],
        capture_output=True,
        text=True,
    )


def test_internal_docker_distribution_is_rejected_before_path_resolution():
    result = run_launcher("-LinuxDistribution", "docker-desktop", "-Frames", "3")
    assert result.returncode == 2
    assert "no Linux distribution is installed" in result.stderr


def test_launcher_contract_contains_real_wsl_execution_boundary():
    text = LAUNCHER.read_text(encoding="utf-8")
    assert "wsl.exe --distribution" in text
    assert "wslpath" in text
    assert "bash -lc" in text
    assert "PRISM_PROTOCOL" in text
    assert "--frames $Frames" in text
    assert "--batch-size $BatchSize" in text
    assert "--evidence-dir $evidenceArg" in text
