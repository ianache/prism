from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_wsl_launcher_builds_bundle_after_gate():
    text = (ROOT / "scripts" / "run_production_gate.ps1").read_text(encoding="utf-8")
    assert "build_evidence_bundle.py" in text
    assert "--source wsl2" in text
    assert "evidence-bundle.json" in text
    assert "--commit" in text
    assert "--digest" in text
