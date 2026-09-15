from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs" / "evidence" / "rust-s23-wsl2-local-gate-enablement-2026-09-15.md"
S22_EVIDENCE = ROOT / "docs" / "evidence" / "rust-s22-local-linux-production-gate-2026-09-15.md"


def test_s23_evidence_contains_complete_four_case_matrix():
    text = EVIDENCE.read_text(encoding="utf-8")
    for case in ("tcp smoke", "http smoke", "tcp 100000", "http 100000"):
        assert f"| {case} |" in text
    for field in ("Comando", "Estado", "Duración", "frames", "usuario efectivo", "digest"):
        assert field.lower() in text.lower()
    assert text.count("NO EJECUTADA") >= 4
    assert "100000" in text


def test_s23_evidence_records_environment_and_sanitization():
    text = EVIDENCE.read_text(encoding="utf-8")
    for marker in ("WSL", "Docker", "Compose", "docker-desktop-data", "GitHub Actions"):
        assert marker in text
    assert "PRISM_AUTH_TOKEN" not in text
    assert "BEGIN PRIVATE KEY" not in text
    assert "s18-ephemeral-token" not in text
    assert "10001:10001" in text


def test_s22_evidence_links_wsl2_follow_up():
    text = S22_EVIDENCE.read_text(encoding="utf-8")
    assert "s23" in text.lower()
